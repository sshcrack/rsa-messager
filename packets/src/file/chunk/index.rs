use anyhow::anyhow;

use openssl::pkey::{Public, Private};
use openssl::rsa::Rsa;
use uuid::Uuid;

use crate::encryption::sign::validate_signature;
use crate::other::key_iv::KeyIVPair;
use crate::util::tools::u64_from_vec;
use crate::util::{tools::{usize_from_vec, uuid_from_vec}, vec::extract_vec};


#[derive(Debug, Clone)]
pub struct ChunkMsg {
    pub signature: Vec<u8>,
    pub encrypted: Vec<u8>,
    pub key: KeyIVPair,
    pub uuid: Uuid,
    pub chunk_index: u64
}


impl ChunkMsg {
    pub fn serialize(&self, receiver_key: &Rsa<Public>) -> anyhow::Result<Vec<u8>> {
        let mut merged = Vec::new();
        let mut b_encrypted = self.encrypted.clone();
        let mut b_key = self.key.serialize(receiver_key)?;

        let signature_size = self.signature.len().to_le_bytes().to_vec();
        let mut b_uuid = self.uuid.clone().as_bytes().to_vec();

        let mut b_chunk_index = self.chunk_index.clone().to_le_bytes().to_vec();

        merged.append(&mut signature_size.clone());
        merged.append(&mut self.signature.clone());
        merged.append(&mut b_uuid);
        merged.append(&mut b_chunk_index);
        merged.append(&mut b_key);
        merged.append(&mut b_encrypted);

        return Ok(merged);
    }

    pub fn deserialize(data: &Vec<u8>, sender_pubkey: &Rsa<Public>, receiver_key: &Rsa<Private>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let signature_size = usize_from_vec(&mut data)?;
        let signature = extract_vec(0..signature_size, &mut data)?;

        let uuid = uuid_from_vec(&mut data)?;
        let chunk_index = u64_from_vec(&mut data)?;

        let key = KeyIVPair::deserialize_mut(&mut data, receiver_key)?;

        let valid = validate_signature(&data, &signature, sender_pubkey)?;
        if !valid {
            return Err(anyhow!("Invalid signature in ChunkByteMsg"));
        }

        return Ok(ChunkMsg {
            signature,
            encrypted: data,
            uuid,
            chunk_index,
            key
        });
    }
}