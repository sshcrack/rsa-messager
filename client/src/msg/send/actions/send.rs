use std::{path::Path, fs::File};

use log::trace;
use colored::Colorize;
use packets::{file::{question::index::FileQuestionMsg, types::FileInfo}, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{util::{tools::uuid_to_name, arcs::{get_receiver, get_curr_id}, msg::{send_msg, print_from_msg}, consts::PENDING_FILES}};

pub async fn on_send(line: &str) -> anyhow::Result<()> {
    let filename = line.split(" ");

    let given_path = Vec::from_iter(filename.skip(1)).join(" ");
    let given_path = Path::new(&given_path);

    if !given_path.is_file() {
        let msg = format!("File '{}' does not exist.", given_path.to_str().unwrap_or("()"));
        println!("{}", msg.red());

        return Ok(());
    }

    let file = File::open(&given_path)?;
    let size = file.metadata()?.len();


    let receiver = get_receiver().await?;
    let receiver_name = uuid_to_name(receiver).await?;


    let curr_id = get_curr_id().await?;
    let uuid = Uuid::new_v4();

    let filename = given_path.file_name();
    if filename.is_none() {
        eprintln!("Could not get filename of path {:?}", given_path.as_os_str());
        return Ok(());
    }

    let filename = filename.unwrap().to_str().unwrap();
    let filename = filename.to_string();

    let to_send = FileQuestionMsg {
        filename: filename.clone(),
        sender: curr_id,
        receiver,
        uuid,
        size
    }.serialize();

    send_msg(Message::Binary(to_send)).await?;
    print_from_msg(&"you".on_bright_red(), &format!("Sending file request to {}", receiver_name.yellow()));

    let info = FileInfo {
        filename: filename.clone(),
        sender: curr_id,
        receiver,
        size,
        path: Some(given_path.to_path_buf())
    };

    trace!("Storing file info {:#?}", info);

    let mut state = PENDING_FILES.write().await;
    state.insert(uuid, info);

    drop(state);

    return Ok(());
}
