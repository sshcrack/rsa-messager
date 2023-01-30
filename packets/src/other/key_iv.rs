use openssl::{rand, symm::{encrypt, decrypt}, pkey::{Private, Public}, rsa::Rsa};

use crate::{
    consts::{AES_DIGEST, AES_IVSIZE_BYTES, AES_KEYSIZE_BYTES, AES_KEYSIZE_BITS, AES_IVSIZE_BITS},
    util::{vec::extract_vec, rsa::{encrypt_rsa, decrypt_rsa}, tools::usize_from_vec},
};
use log::trace;

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
        let mut b_key_size = b_key.len().to_le_bytes().to_vec();

        let mut b_iv = encrypt_rsa(keypair, &self.iv.clone())?;
        let mut b_iv_size = b_iv.len().to_le_bytes().to_vec();

        trace!("Serializing KeyIVPair with key size {} and iv size {}", b_key.len(), b_iv.len());
        merged.append(&mut b_key_size);
        merged.append(&mut b_key);
        merged.append(&mut b_iv_size);
        merged.append(&mut b_iv);

        return Ok(merged);
    }

    pub fn deserialize_mut(data: &mut Vec<u8>, keypair: &Rsa<Private>) -> anyhow::Result<Self>
    where Self: Sized {
        let key_size = usize_from_vec(data)?;
        let key = extract_vec(0..key_size, data)?;

        let iv_size = usize_from_vec(data)?;
        let iv = extract_vec(0..iv_size, data)?;

        let key = decrypt_rsa(keypair, &key)?;
        let iv = decrypt_rsa(keypair, &iv)?;

        trace!("Deserializing KeyIVPair with key size {} and iv size {}", key_size, iv_size);
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
        let encrypted = encrypt(*AES_DIGEST, &self.key, Some(&self.iv), data)?;

        return Ok(encrypted);
    }

    pub fn decrypt(&self, data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let decrypted =decrypt(*AES_DIGEST, &self.key, Some(&self.iv), data)?;

        return Ok(decrypted);
    }
}
