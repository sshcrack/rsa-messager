use log::trace;
use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};

use crate::{types::WSMessage, util::modes::Modes};

pub struct ErrorMsg {
    pub error: String
}

impl WSMessage for ErrorMsg {
    fn serialize(&self) -> Vec<u8> {
        return Modes::Error.get_send(&self.error.as_bytes().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();

        trace!("Parsing error of length {}...", data.len());
        let error = String::from_utf8(data)?;
        RsaPublicKey::from_public_key_pem(&error)?;

        return Ok(ErrorMsg {
            error
        });
    }
}