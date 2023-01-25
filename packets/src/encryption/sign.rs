use openssl::{rsa::Rsa, pkey::{Private, PKey, Public}, sign::{Signer, Verifier}};

use crate::consts::MSG_DIGEST;

pub fn get_signature(data: &Vec<u8>, keypair: &Rsa<Private>) -> anyhow::Result<Vec<u8>>{
    let p_key = PKey::from_rsa(keypair.clone())?;

    let mut signer = Signer::new(*MSG_DIGEST, &p_key)?;
    signer.update(&data)?;

    return Ok(signer.sign_to_vec()?);
}

pub fn verify_data(data: &Vec<u8>, signature: &Vec<u8>, pubkey: &Rsa<Public>) -> anyhow::Result<bool> {
    let p_key = PKey::from_rsa(pubkey.clone())?;
    let mut verifier = Verifier::new(*MSG_DIGEST, &p_key)?;
    verifier.update(&data)?;

    let valid = verifier.verify(&signature)?;
    return Ok(valid);
}