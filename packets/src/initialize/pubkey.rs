use log::trace;
use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};

use crate::{types::WSMessage, util::modes::Modes};

pub struct PubkeyMsg {
    pub pubkey: String
}

impl WSMessage for PubkeyMsg {
    fn serialize(&self) -> Vec<u8> {
        return Modes::SetPubkey.get_send(&self.pubkey.as_bytes().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();

        trace!("Parsing public key of length {}...", data.len());
        let pubkey = String::from_utf8(data)?;
        RsaPublicKey::from_public_key_pem(&pubkey)?;

        return Ok(PubkeyMsg {
            pubkey
        });
    }
}