use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};

pub fn encrypt_rsa(key: &Rsa<Public>, msg: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0; key.size() as usize];

    let size = key.public_encrypt(&msg, &mut buf, Padding::PKCS1_OAEP)?;

    let mut out = vec![0; size];
    for i in 0..size {
        out[i] = buf[i];
    }

    return Ok(out);
}

pub fn decrypt_rsa(key: &Rsa<Private>, cipher: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0; key.size() as usize];

    let size = key.private_decrypt(cipher, &mut buf, Padding::PKCS1_OAEP)?;

    let mut out = vec![0; size];
    for i in 0..size {
        out[i] = buf[i];
    }

    return Ok(out);
}