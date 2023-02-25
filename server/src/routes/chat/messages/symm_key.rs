use packets::{communication::key_reply_encrypted::EncryptedSymmKeyReplyMsg, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::tools::send_msg_specific;


pub async fn on_symm_key(data: Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let EncryptedSymmKeyReplyMsg { user: receiver, key } = EncryptedSymmKeyReplyMsg::deserialize(&data)?;
    
    let packet = EncryptedSymmKeyReplyMsg {
        user: my_id.clone(),
        key
    }.serialize();

    send_msg_specific(receiver, Message::binary(packet)).await?;
    Ok(())
}
