use std::{path::Path, collections::VecDeque};

use colored::Colorize;
use tokio_tungstenite::tungstenite::Message;

use crate::{util::{tools::uuid_to_name, vec::{decque_to_vec, vec_to_decque}, arcs::{get_receiver, get_curr_id}, msg::{send_msg, print_from_msg}}, msg::parsing::{file::question_client::FileQuestionClientMsg, types::WSMessage}};

pub async fn on_send(line: &str) -> anyhow::Result<()> {
    let filename = line.split(" ");
    let filename = Vec::from_iter(filename.skip(1)).join(" ");
    let p = Path::new(&filename);

    if !p.exists() {
        let msg = format!("File {} does not exist.", filename);
        println!("{}", msg.red());

        return Ok(());
    }


    let receiver = get_receiver().await?;
    let receiver_name = uuid_to_name(receiver).await?;


    let curr_id = get_curr_id().await?;

    let to_send = FileQuestionClientMsg {
        filename,
        sender: curr_id,
        receiver
    }.serialize();

    send_msg(Message::Binary(to_send)).await?;
    print_from_msg(&"you".on_bright_red(), &format!("Sending file request to {}", receiver_name.yellow()));

    return Ok(());
}
