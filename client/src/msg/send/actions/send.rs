use std::{path::Path, fs::File};

use log::trace;
use colored::Colorize;
use packets::{file::{question::index::FileQuestionMsg, types::FileInfo}, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{util::{tools::uuid_to_name, arcs::{get_receiver, get_curr_id}, msg::{send_msg, print_from_msg}, consts::PENDING_FILES}};

pub async fn on_send(line: &str) -> anyhow::Result<()> {
    let filename = line.split(" ");
    let filename = Vec::from_iter(filename.skip(1)).join(" ");
    let p = Path::new(&filename);

    if !p.is_file() {
        let msg = format!("File {} does not exist.", filename);
        println!("{}", msg.red());

        return Ok(());
    }

    let file = File::open(&filename)?;
    let size = file.metadata()?.len();


    let receiver = get_receiver().await?;
    let receiver_name = uuid_to_name(receiver).await?;


    let curr_id = get_curr_id().await?;
    let uuid = Uuid::new_v4();

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
        filename,
        sender: curr_id,
        receiver,
        size
    };

    trace!("Storing file info {:#?}", info);

    let mut state = PENDING_FILES.write().await;
    state.insert(uuid, info);

    drop(state);

    return Ok(());
}
