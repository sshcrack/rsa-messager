use colored::Colorize;
use packets::{
    file::{
        question::{index::FileQuestionMsg, reply::FileQuestionReplyMsg},
        types::FileInfo,
    },
    types::WSMessage,
};
use pretty_bytes::converter::convert;
use tokio_tungstenite::tungstenite::Message;

use crate::util::{
    consts::PENDING_FILES,
    msg::send_msg,
    tools::{uuid_to_name, wait_confirm},
};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileQuestionMsg {
        filename,
        receiver,
        sender,
        uuid,
        size,
        secret,
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
        let allowed = format!("Receiving '{}'{}", filename.bright_green(), "...".yellow());
        println!("{}", allowed.green());

        let info = FileInfo {
            filename,
            receiver,
            sender,
            size,
            secret,
        };

        let mut state = PENDING_FILES.write().await;
        state.insert(uuid, info);

        drop(state);
    }

    let to_send = FileQuestionReplyMsg { accepted, uuid }.serialize();
    send_msg(Message::binary(to_send)).await?;
    Ok(())
}
