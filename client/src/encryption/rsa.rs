use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};
use std::str;

pub fn encrypt(key: Rsa<Public>, msg: &str) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0; key.size() as usize];

    let size = key.public_encrypt(&msg.as_bytes(), &mut buf, Padding::PKCS1_OAEP)?;

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
