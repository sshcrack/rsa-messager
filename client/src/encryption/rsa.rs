use anyhow::anyhow;
use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};
use uuid::Uuid;

use crate::web::user_info::get_user_info;

pub fn encrypt(key: Rsa<Public>, msg: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0; key.size() as usize];

    let size = key.public_encrypt(&msg, &mut buf, Padding::PKCS1_OAEP)?;

    let mut out = vec![0; size];
    for i in 0..size {
        out[i] = buf[i];
    }

    return Ok(out);
}

pub fn decrypt(key: Rsa<Private>, cipher: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0; key.size() as usize];

    let size = key.private_decrypt(&cipher, &mut buf, Padding::PKCS1_OAEP)?;

    let mut out = vec![0; size];
    for i in 0..size {
        out[i] = buf[i];
    }

    return Ok(out);
}

pub fn generate() -> Rsa<Private> {
    let rsa = Rsa::generate(2048).unwrap();

    return rsa;
}

pub async fn get_pubkey_from_rec(rec: &Uuid) -> anyhow::Result<Rsa<Public>>{
    let info = get_user_info(rec).await?;
    let pubkey_pem = info.public_key;

    if pubkey_pem.is_none() {
        println!("Could not get pubkey of receiver.");
        return Err(anyhow!("Could not get pubkey of receiver."));
    }

    let pubkey_pem = pubkey_pem.unwrap();
    let key = Rsa::public_key_from_pem(pubkey_pem.as_bytes())?;

    Ok(key)
}