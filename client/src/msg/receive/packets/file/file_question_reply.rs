use colored::Colorize;
use packets::{file::question::reply::FileQuestionReplyMsg, types::WSMessage};

use crate::util::tools::uuid_to_name;

pub async fn on_file_question_reply(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = FileQuestionReplyMsg::deserialize(data)?;

    let sender = msg.sender;
    let sender_name = uuid_to_name(sender).await?;

    let filename = msg.filename;
    let accepted = msg.accepted;

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
