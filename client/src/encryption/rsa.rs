use anyhow::anyhow;
use openssl::pkey::{Private, Public};
use openssl::rsa::Rsa;
use packets::consts::RSA_KEY_BITS;
use uuid::Uuid;

use crate::web::user_info::get_user_info;

pub fn generate() -> Rsa<Private> {
    let rsa = Rsa::generate(RSA_KEY_BITS).unwrap();

    return rsa;
}

pub async fn get_pubkey_from_rec(rec: &Uuid) -> anyhow::Result<Rsa<Public>>{
    let info = get_user_info(rec).await?;
    let key = info.public_key;

    if key.is_none() {
        println!("Could not get pubkey of receiver.");
        return Err(anyhow!("Could not get pubkey of receiver."));
    }

    let key = key.unwrap();
    Ok(key)
}