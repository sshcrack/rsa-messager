use anyhow::anyhow;
use openssl::{rsa::Rsa, pkey::Private};
use uuid::Uuid;

use super::consts::{RECEIVER, CURR_ID, KEYPAIR};


pub async fn get_curr_keypair() -> anyhow::Result<Rsa<Private>> {
    let state = KEYPAIR.read().await;
    let keypair = state.clone();

    drop(state);
    if keypair.is_none() {
        return Err(anyhow!("Keypair not generated."));
    }

    return Ok(keypair.unwrap());
}

#[allow(unused)]  // TODO
pub async fn get_curr_id() -> anyhow::Result<Uuid> {
    let state = CURR_ID.read().await;
    let curr_id = state.clone();

    drop(state);
    if curr_id.is_none() {
        return Err(anyhow!("Current id is null."));
    }

    return Ok(curr_id.unwrap());
}

pub async fn get_receiver() -> anyhow::Result<Uuid> {
    let state = RECEIVER.read().await;
    let curr_id = state.clone();

    drop(state);
    if curr_id.is_none() {
        return Err(anyhow!("Receiver is null."));
    }

    return Ok(curr_id.unwrap());
}