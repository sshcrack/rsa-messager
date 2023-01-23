use colored::Colorize;
use packets::{file::{question_server::FileQuestionServerMsg, reply::FileQuestionReplyMsg}, types::WSMessage};
use tokio_tungstenite::tungstenite::Message;

use crate::util::{msg::{get_input, send_msg}, tools::uuid_to_name};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileQuestionServerMsg { filename,receiver, sender, uuid} = FileQuestionServerMsg::deserialize(data)?;
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

    let to_send = FileQuestionReplyMsg {
        accepted,
        filename,
        receiver,
        sender,
        uuid
    }.serialize();

    send_msg(Message::binary(to_send)).await?;
    Ok(())
}
