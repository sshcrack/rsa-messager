use std::path::PathBuf;

use anyhow::anyhow;
use colored::Colorize;
use log::trace;
use openssl::rsa::Rsa;
use packets::{
    file::{
        question::{ index::FileQuestionMsg, reply::FileQuestionReplyMsg },
        types::FileInfo,
        processing::tools::get_max_chunks,
    },
    types::ByteMessage,
};
use pretty_bytes::converter::convert;
use tokio::{ fs::{File, remove_file}, io::AsyncWriteExt };
use tokio_tungstenite::tungstenite::Message;

use crate::{
    util::{
        consts::{ PENDING_FILES, FILE_DOWNLOADS },
        msg::{ send_msg, get_input },
        tools::{ uuid_to_name, wait_confirm },
    },
    file::downloader::index::Downloader,
    web::user_info::get_user_info,
};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = FileQuestionMsg::deserialize(data)?;
    let sender_name = uuid_to_name(msg.sender).await?;

    let size_str = convert(msg.size as f64);
    let confirm_msg = format!(
        "{} wants to send you the file '{}' of size {}. Accept? (y/n)",
        sender_name.green(),
        msg.filename.yellow(),
        size_str.purple()
    );
    println!("{}", confirm_msg);

    let accepted = check_accepted(msg.clone()).await?;
    let to_send = (FileQuestionReplyMsg {
        accepted,
        uuid: msg.uuid.clone(),
    }).serialize();

    send_msg(Message::binary(to_send)).await?;
    Ok(())
}

pub async fn check_accepted(msg: FileQuestionMsg) -> anyhow::Result<bool> {
    let FileQuestionMsg { filename, receiver, size, sender, uuid } = msg;

    let accepted = wait_confirm().await?;
    if !accepted {
        let denied = format!("You denied '{}' file request.", filename.bright_red());
        println!("{}", denied.red());
        return Ok(false);
    }

    let path: Option<PathBuf>;
    loop {
        println!(
            "{}",
            format!(
                "Where do you want to save this file (default is in current directory)?"
            ).yellow()
        );
        let raw_path = get_input().await?;
        let mut try_path = PathBuf::from(&raw_path);
        if try_path.try_exists()? {
            println!(
                "{}",
                format!(
                    "This file exists already. Overwrite? ({}/{})",
                    "y".green(),
                    "n".red()
                ).yellow()
            );

            let overwrite = wait_confirm().await?;
            if overwrite {
                println!("{}", format!("Alright, overwriting file."));

                path = Some(try_path.to_path_buf());
                break;
            }
            continue;
        }

        if try_path.is_dir() {
            let c = try_path.clone();
            try_path = c.join(&filename);
        }

        let f = File::create(try_path.clone()).await;
        if f.is_err() {
            let err = f.unwrap_err();
            eprintln!(
                "{}",
                format!(
                    "Could not create file at given path: {:?}. Please enter a valid path.",
                    err
                ).red()
            );
            continue;
        }

        let mut f = f.unwrap();
        f.shutdown().await?;

        remove_file(try_path.clone()).await?;
        path = Some(try_path);
        break;
    }

    if path.is_none() {
        eprintln!("Invalid path has been selected. Aborting...");
        return Ok(false);
    }

    let path = path.unwrap();
    let allowed = format!(
        "Receiving '{}'{} (uuid: {})",
        filename.bright_green(),
        "...".yellow(),
        uuid
    );
    println!("{}", allowed.green());

    let info = FileInfo {
        filename,
        receiver,
        sender,
        size,
        path: Some(path),
    };

    trace!("Waiting for pending files...");
    let mut state = PENDING_FILES.write().await;
    state.insert(uuid, info.clone());

    drop(state);

    trace!("Getting user info...");
    let user = get_user_info(&sender).await?;
    let key = user.public_key;

    if key.is_none() {
        return Err(anyhow!("Sender does not have a public key"));
    }

    let key = key.unwrap();
    let key = Rsa::public_key_from_pem(key.as_bytes())?;

    trace!("Initializing downloader...");
    let mut downloader = Downloader::new(&uuid, key, &info);
    downloader.initialize(get_max_chunks(info.size)).await?;

    trace!("Aquiring lock on file_downloads...");
    let mut state = FILE_DOWNLOADS.write().await;

    trace!("Inserting downloader into FILE_DOWNLOADS...");
    state.insert(uuid, downloader);

    drop(state);
    trace!("Done.");
    return Ok(true);
}