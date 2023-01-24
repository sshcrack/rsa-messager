use colored::Colorize;
use packets::{file::{question::reply::FileQuestionReplyMsg, types::FileInfo}, types::WSMessage};

use crate::{util::tools::uuid_to_name, file::tools::get_pending_file};

pub async fn on_file_question_reply(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileQuestionReplyMsg { accepted, uuid} = FileQuestionReplyMsg::deserialize(data)?;
    let file = get_pending_file(uuid).await;

    if file.is_err() {
        eprintln!("{}", format!("Could not receive file. Invalid UUID: {}", uuid).red());
        return Ok(());
    }

    let FileInfo { sender, filename,.. } = file.unwrap();
    let sender_name = uuid_to_name(sender).await?;

    if accepted {
        println!("{}", format!(
            "{} {} your file request of file '{}'.",
            sender_name.bright_blue(),
            "accepted".green(),
            filename.yellow()
        ));
    } else {
        println!("{}", format!(
            "{} {} your file request of file '{}'.",
            sender_name.bright_blue(),
            "rejected".on_red(),
            filename.yellow()
        ))
    }
    Ok(())
}
