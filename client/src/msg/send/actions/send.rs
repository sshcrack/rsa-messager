use std::{path::Path, collections::VecDeque};

use colored::Colorize;
use tokio_tungstenite::tungstenite::Message;

use crate::util::{modes::Modes, tools::uuid_to_name, vec::{decque_to_vec, vec_to_decque}, arcs::get_receiver, msg::{send_msg, print_from_msg}};

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


    let mut merged = VecDeque::new();
    let b_receiver = receiver.as_bytes().to_vec();
    let mut b_receiver = vec_to_decque(b_receiver);

    let b_filename = filename.as_bytes().to_vec();
    let mut b_filename = vec_to_decque(b_filename);

    merged.append(&mut b_receiver);
    merged.append(&mut b_filename);

    let merged = decque_to_vec(merged);
    let to_send = Modes::SendFileQuestion.get_send(&merged);

    send_msg(Message::Binary(to_send)).await?;
    print_from_msg(&"you".on_bright_red(), &format!("Sending file request to {}", receiver_name.yellow()));

    return Ok(());
}
