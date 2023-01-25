use log::trace;
use openssl::rsa::Rsa;

use crate::{types::ByteMessage, util::modes::Modes};

pub struct PubkeyMsg {
    pub pubkey: String
}

impl ByteMessage for PubkeyMsg {
    fn serialize(&self) -> Vec<u8> {
        return Modes::SetPubkey.get_send(&self.pubkey.as_bytes().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();

        trace!("Parsing public key of length {}...", data.len());
        Rsa::public_key_from_pem(&data)?;

        let pubkey = String::from_utf8(data)?;
        return Ok(PubkeyMsg {
            pubkey
        });
    }
}