use packets::{communication::{to::ToMsg, from::FromMsg}, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::tools::send_msg_specific;


pub async fn on_to(data: Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let ToMsg { msg, receiver} = ToMsg::deserialize(&data)?;
    let packet = FromMsg {
        msg,
        sender: my_id.clone()
    }.serialize();

    send_msg_specific(receiver, Message::binary(packet)).await?;

    Ok(())
}
