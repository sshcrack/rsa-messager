use packets::{communication::{to::ToMsg, from::FromMsg}, types::WSMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{types::Users, tools::send_msg_specific};


pub async fn on_to(data: Vec<u8>, my_id: &Uuid, users: &Users) -> anyhow::Result<()> {
    let ToMsg { msg, receiver} = ToMsg::deserialize(&data)?;
    let packet = FromMsg {
        msg,
        sender: my_id.clone()
    }.serialize();

    send_msg_specific(receiver, users, Message::binary(packet)).await?;

    Ok(())
}
