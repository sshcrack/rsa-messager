use log::trace;
use openssl::{rsa::Rsa, pkey::{Public, Private}};

use crate::{types::ByteMessage, util::modes::Modes};

pub struct PubkeyMsg {
    pub pubkey: Rsa<Public>
}

impl PubkeyMsg {
    pub fn from_private(privkey: Rsa<Private>) -> anyhow::Result<Self> {
        let pem = privkey.public_key_to_pem()?;
        let pubkey = Rsa::public_key_from_pem(&pem)?;

        return Ok(PubkeyMsg {
            pubkey
        })
    }
}

impl ByteMessage for PubkeyMsg {

    fn serialize(&self) -> Vec<u8> {
        return Modes::SetPubkey.get_send(&self.pubkey.public_key_to_pem().unwrap().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();

        trace!("Parsing public key of length {}...", data.len());
        let key = Rsa::public_key_from_pem(&data)?;

        return Ok(PubkeyMsg {
            pubkey: key
        });
    }
}