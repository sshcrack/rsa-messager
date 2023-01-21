use std::collections::VecDeque;

use anyhow::anyhow;
use openssl::{rsa::Rsa, pkey::Private};
use uuid::Uuid;

use super::consts::{UUID_SIZE, KEYPAIR};

pub fn vec_to_decque<T>(v: Vec<T>) -> VecDeque<T> {
    let mut r: VecDeque<T> = VecDeque::new();
    for i in v {
        r.push_back(i);
    }

    return r;
}

pub fn decque_to_vec<T>(v: VecDeque<T>) -> Vec<T> {
    let mut r: Vec<T> = Vec::new();
    for i in v {
        r.push(i);
    }

    return r;
}

pub fn bytes_to_uuid(v: &Vec<u8>) -> anyhow::Result<Uuid> {
    if v.len() != UUID_SIZE {
        return Err(anyhow!(format!(
            "Invalid length of uuid. Length is {}",
            v.len()
        )));
    }

    let mut buff: [u8; 16] = [0; 16];
    for i in 0..UUID_SIZE {
        buff[i] = v.get(i).unwrap().to_owned();
    }

    let uuid = Uuid::from_bytes(buff);
    return Ok(uuid);
}

pub fn uuid_from_vec(v: &mut Vec<u8>) -> anyhow::Result<Uuid> {
    let uuid = v.splice(0..16, vec![]);
    let uuid = Vec::from_iter(uuid);

    let uuid = bytes_to_uuid(&uuid)?;
    return Ok(uuid);
}

pub async fn get_curr_keypair() -> anyhow::Result<Rsa<Private>> {
    let state = KEYPAIR.read().await;
    let keypair = state.clone();

    drop(state);
    if keypair.is_none() {
        return Err(anyhow!("Keypair not generated."));
    }

    return Ok(keypair.unwrap());
}