use anyhow::anyhow;
use colored::Colorize;
use log::trace;
use openssl::rsa::Rsa;
use packets::{
    file::{
        question::{index::FileQuestionMsg, reply::FileQuestionReplyMsg},
        types::FileInfo, processing::tools::get_max_threads,
    },
    types::ByteMessage,
};
use pretty_bytes::converter::convert;
use tokio_tungstenite::tungstenite::Message;

use crate::{util::{
    consts::{PENDING_FILES, FILE_DOWNLOADS},
    msg::send_msg,
    tools::{uuid_to_name, wait_confirm},
}, file::downloader::index::Downloader, web::user_info::get_user_info};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileQuestionMsg {
        filename,
        receiver,
        sender,
        uuid,
        size
    } = FileQuestionMsg::deserialize(data)?;
    let sender_name = uuid_to_name(sender).await?;

    let size_str = convert(size as f64);
    let confirm_msg = format!(
        "{} wants to send you the file '{}' of size {}. Accept? (y/n)",
        sender_name.green(),
        filename.yellow(),
        size_str.purple()
    );
    println!("{}", confirm_msg);

    let accepted = wait_confirm().await?;
    if !accepted {
        let denied = format!("You denied '{}' file request.", filename.bright_red());
        println!("{}", denied.red())
    } else {
        let allowed = format!("Receiving '{}'{} (uuid: {})", filename.bright_green(), "...".yellow(), uuid);
        println!("{}", allowed.green());

        let info = FileInfo {
            filename,
            receiver,
            sender,
            size
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
        downloader.initialize(get_max_threads(info.size)).await?;

        trace!("Aquiring lock on file_downloads...");
        let mut state = FILE_DOWNLOADS.write().await;

        trace!("Inserting downloader into FILE_DOWNLOADS...");
        state.insert(uuid, downloader);

        drop(state);
    }

    let to_send = FileQuestionReplyMsg { accepted, uuid }.serialize();
    send_msg(Message::binary(to_send)).await?;
    Ok(())
}
