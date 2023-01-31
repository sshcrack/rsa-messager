use anyhow::anyhow;
use openssl::pkey::{Private, Public};
use openssl::rsa::Rsa;
use uuid::Uuid;

use crate::web::user_info::get_user_info;

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