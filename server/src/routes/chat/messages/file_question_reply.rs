use packets::{file::reply::FileQuestionReplyMsg, types::WSMessage};
use warp::ws::Message;

use crate::utils::{
    tools::send_msg_specific,
    types::Users,
};

pub async fn on_file_question_reply(
    data: &Vec<u8>,
    users: &Users
) -> anyhow::Result<()> {
    let msg = FileQuestionReplyMsg::deserialize(&data)?;
    let to_send = msg.serialize();

    send_msg_specific(msg.receiver, users, Message::binary(to_send)).await?;
    Ok(())
}
