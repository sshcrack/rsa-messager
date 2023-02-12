use openssl::{rand, symm::{encrypt, decrypt}, pkey::{Private, Public}, rsa::Rsa};

use crate::{
    consts::{AES_DIGEST, AES_IVSIZE_BYTES, AES_KEYSIZE_BYTES, AES_KEYSIZE_BITS, AES_IVSIZE_BITS},
    util::{vec::extract_vec, rsa::{encrypt_rsa, decrypt_rsa}, tools::{vec_to_usize, usize_to_vec}},
};

#[derive(Debug, Clone)]
pub struct KeyIVPair {
    pub key: Vec<u8>,
    pub iv: Vec<u8>,
}

impl KeyIVPair {
    pub const BYTE_SIZE: usize = AES_KEYSIZE_BYTES + AES_IVSIZE_BYTES;
    pub const BIT_SIZE: usize = AES_KEYSIZE_BITS + AES_IVSIZE_BITS;

    pub fn serialize(&self, keypair: &Rsa<Public>) -> anyhow::Result<Vec<u8>> {
        let mut merged = Vec::new();
        let mut b_key = encrypt_rsa(keypair, &self.key.clone())?;
        let mut b_key_size = usize_to_vec(b_key.len())?;

        let mut b_iv = encrypt_rsa(keypair, &self.iv.clone())?;
        let mut b_iv_size = usize_to_vec(b_iv.len())?;

        merged.append(&mut b_key_size);
        merged.append(&mut b_key);
        merged.append(&mut b_iv_size);
        merged.append(&mut b_iv);

        return Ok(merged);
    }

    pub fn deserialize_mut(data: &mut Vec<u8>, keypair: &Rsa<Private>) -> anyhow::Result<Self>
    where Self: Sized {
        let key_size = vec_to_usize(data)?;
        let key = extract_vec(0..key_size, data)?;

        let iv_size = vec_to_usize(data)?;
        let iv = extract_vec(0..iv_size, data)?;

        let key = decrypt_rsa(keypair, &key)?;
        let iv = decrypt_rsa(keypair, &iv)?;

        return Ok(Self { key, iv });
    }

    pub fn deserialize(data: &Vec<u8>, keypair: &Rsa<Private>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut data = data.clone();
        return Self::deserialize_mut(&mut data, keypair);
    }
}

impl KeyIVPair {
    pub fn generate() -> anyhow::Result<Self> {
        let mut key = [0 as u8; AES_KEYSIZE_BYTES];
        let mut iv = [0 as u8; AES_IVSIZE_BYTES];

        rand::rand_bytes(&mut key)?;
        rand::rand_bytes(&mut iv)?;

        return Ok(Self {
            key: key.to_vec(),
            iv: iv.to_vec(),
        });
    }

    pub fn encrypt(&self, data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
        println!("Encrypting dataLen: {} with iv {} and key {}", data.len(), self.iv.len(), self.key.len());
        let encrypted = encrypt(*AES_DIGEST, &self.key, Some(&self.iv), data)?;

        println!("Done encrypting");
        return Ok(encrypted);
    }

    pub fn decrypt(&self, data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let decrypted =decrypt(*AES_DIGEST, &self.key, Some(&self.iv), data)?;

        return Ok(decrypted);
    }
}
