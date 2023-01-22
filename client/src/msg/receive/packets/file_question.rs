use std::collections::VecDeque;

use colored::Colorize;
use tokio_tungstenite::tungstenite::Message;

use crate::util::{
    modes::Modes,
    msg::{get_input, send_msg},
    tools::{uuid_from_vec, uuid_to_name}, vec::{vec_to_decque, decque_to_vec},
};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let sender = uuid_from_vec(data)?;
    let filename = String::from_utf8(data.to_owned())?;

    let sender_name = uuid_to_name(sender).await?;

    let confirm_msg = format!(
        "{} wants to send you the file '{}'. Accept? (y/n)",
        sender_name.green(),
        filename.yellow()
    );
    println!("{}", confirm_msg);

    let accepted:bool;
    loop {
        let input = get_input().await?;

        let input = input.to_lowercase();
        if !input.eq("y") && !input.eq("n") {
            eprintln!("You have to enter either y/n");
            continue;
        }

        accepted = input.eq("y");
        break;
    }

    if !accepted {
        let denied = format!("You denied '{}' file request.", filename.bright_red());
        println!("{}", denied.red())
    } else {
        let allowed = format!("Receiving '{}'{}", filename.bright_green(), "...".yellow());
        println!("{}", allowed.green());
    }

    let accepted_vec: u8 = if accepted == true { 0 } else { 1 };

    let mut merged: VecDeque<u8> = VecDeque::new();
    let mut b_filename = vec_to_decque(filename.clone().as_bytes().to_vec());
    let mut b_uuid = vec_to_decque(sender.as_bytes().to_vec());

    merged.append(&mut b_uuid);
    merged.push_back(accepted_vec);
    merged.append(&mut b_filename);

    let to_send = Modes::SendFileQuestionReply.get_send(&decque_to_vec(merged));

    send_msg(Message::binary(to_send)).await?;
    Ok(())
}
