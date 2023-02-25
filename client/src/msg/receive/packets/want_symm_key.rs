use packets::{communication::{key_request::WantSymmKeyMsg, key_reply::SymmKeyReplyMsg}, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;

use crate::{util::{msg::send_msg, arcs::get_symm_key_or_default}, encryption::rsa::get_pubkey_from_rec};

pub async fn on_want_symm_key(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let WantSymmKeyMsg { user } =  WantSymmKeyMsg::deserialize(data)?;
    
    let key = get_symm_key_or_default(&user).await?;
    let msg = SymmKeyReplyMsg {
        key,
        user
    };

    let pubkey = get_pubkey_from_rec(&user).await?;

    send_msg(Message::binary(msg.serialize(pubkey)?)).await?;
    Ok(())
}
