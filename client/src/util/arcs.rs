use anyhow::anyhow;
use openssl::{rsa::Rsa, pkey::Private};
use packets::other::key_iv::KeyIVPair;
use uuid::Uuid;

use super::consts::{RECEIVER, CURR_ID, KEYPAIR, BASE_URL, USE_TLS, CONCURRENT_THREADS, CHAT_SYMM_KEYS};


pub async fn get_curr_keypair() -> anyhow::Result<Rsa<Private>> {
    let state = KEYPAIR.read().await;
    let keypair = state.clone();

    drop(state);
    if keypair.is_none() {
        return Err(anyhow!("Keypair not generated."));
    }

    return Ok(keypair.unwrap());
}

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


pub async fn get_base_url() -> String {
    let state = BASE_URL.read().await;
    let base_url = state.clone();

    drop(state);

    return base_url;
}


pub async fn use_tls() -> bool {
    let state = USE_TLS.read().await;
    let out = state.clone();

    return out;
}

pub async fn get_concurrent_threads() -> u64 {
    let state = CONCURRENT_THREADS.read().await;
    let threads = state.clone();

    drop(state);
    return threads;
}

pub async fn get_symm_key(user: &Uuid) -> anyhow::Result<KeyIVPair> {
    let state = CHAT_SYMM_KEYS.read().await;
    let key = state.get(&user).unwrap_or(&(None as Option<KeyIVPair>)).to_owned();

    drop(state);
    if key.is_none() {
        return Err(anyhow!("Could not get symm key"));
    } else {
        return Ok(key.unwrap());
    }
}

pub async fn get_symm_key_or_default(user: &Uuid) -> anyhow::Result<KeyIVPair> {
    let mut state = CHAT_SYMM_KEYS.write().await;
    let key = state.get(&user).unwrap_or(&(None as Option<KeyIVPair>)).to_owned();

    if key.is_none() {
        let key = KeyIVPair::generate()?;
        state.insert(user.clone(), Some(key.clone()));

        return Ok(key);
    } else {
        return Ok(key.unwrap());
    }
}