use log::debug;
use packets::{initialize::pubkey::PubkeyMsg, types::ByteMessage};
use uuid::Uuid;

use crate::{file::consts::USERS};

pub async fn on_pubkey(data: &Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let PubkeyMsg { pubkey } = PubkeyMsg::deserialize(data)?;

    let mut state = USERS.write().await;
    let info = state.get_mut(&my_id);

    if info.is_some() {
        let i = info.unwrap();
        i.public_key = Some(pubkey.clone());
    }

    drop(state);
    debug!("Pubkey set. len: {}", pubkey.len());
    Ok(())
}
