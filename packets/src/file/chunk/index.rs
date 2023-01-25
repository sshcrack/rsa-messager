use anyhow::anyhow;
use log::trace;

use openssl::pkey::Public;
use openssl::rsa::Rsa;
use uuid::Uuid;

use crate::encryption::sign::verify_data;
use crate::util::{tools::{usize_from_vec, uuid_from_vec}, vec::extract_vec};


pub struct ChunkMsg {
    pub signature: Vec<u8>,
    pub encrypted: Vec<u8>,
    pub uuid: Uuid
}

pub trait  ChunkByteMessage {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &Vec<u8>, pubkey: &Rsa<Public>) -> anyhow::Result<Self> where Self: Sized;
}


impl ChunkByteMessage for ChunkMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged = Vec::new();
        let b_encrypted = self.encrypted.clone();

        let signature_size = self.signature.len().to_le_bytes().to_vec();
        let mut b_uuid = self.uuid.clone().as_bytes().to_vec();

        merged.append(&mut signature_size.clone());
        merged.append(&mut self.signature.clone());
        merged.append(&mut b_uuid);
        merged.append(&mut b_encrypted.clone());

        return merged;
    }

    fn deserialize(data: &Vec<u8>, pubkey: &Rsa<Public>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        trace!("Parsing error of length {}...", data.len());
        let signature_size = usize_from_vec(&mut data)?;
        let signature = extract_vec(0..signature_size, &mut data)?;

        let uuid = uuid_from_vec(&mut data)?;

        let valid = verify_data(&data, &signature, pubkey)?;
        if !valid {
            return Err(anyhow!("Invalid signature in ChunkByteMsg"));
        }

        return Ok(ChunkMsg {
            signature,
            encrypted: data,
            uuid
        });
    }
}