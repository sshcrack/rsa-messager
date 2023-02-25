use packets::{communication::key_reply::SymmKeyReplyMsg};

use crate::util::{arcs::get_curr_keypair, consts::CHAT_SYMM_KEYS};

pub async fn on_symm_key(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let key = get_curr_keypair().await?;
    let SymmKeyReplyMsg { user, key: symm_key } =  SymmKeyReplyMsg::deserialize(data, key)?;
    
    let mut state = CHAT_SYMM_KEYS.write().await;
    let waits_for_key = state.get(&user).is_some();

    if waits_for_key {
        state.insert(user.clone(), Some(symm_key));
    }

    drop(state);
    Ok(())
}
