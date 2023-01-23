use log::debug;
use uuid::Uuid;
use warp::ws::Message;

use crate::{utils::{
    tools::send_msg_specific,
    types::Users,
}, msg::parsing::{file::{question_client::FileQuestionClientMsg, question_server::FileQuestionServerMsg}, types::WSMessage}};

pub async fn on_file_question(
    data: &Vec<u8>,
    curr_id: &Uuid,
    users: &Users
) -> anyhow::Result<()> {
    let FileQuestionClientMsg { filename, receiver, sender} = FileQuestionClientMsg::deserialize(&data)?;

    debug!("Sending {} to {}", filename, receiver);
    let msg = FileQuestionServerMsg {
        filename,
        sender,
        receiver,
        uuid: Uuid::new_v4()
    };

    let to_send = msg.serialize();
    send_msg_specific(receiver, users, Message::binary(to_send)).await?;
    Ok(())
}
