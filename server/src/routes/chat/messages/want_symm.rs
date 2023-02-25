use packets::{communication::key_request::WantSymmKeyMsg, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::tools::send_msg_specific;


pub async fn on_want_symm_key(data: Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let WantSymmKeyMsg { user: receiver } = WantSymmKeyMsg::deserialize(&data)?;
    
    let packet = WantSymmKeyMsg {
        user: my_id.clone()
    }.serialize();

    send_msg_specific(receiver, Message::binary(packet)).await?;
    Ok(())
}
