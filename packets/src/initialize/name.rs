use anyhow::anyhow;
use log::trace;

use crate::{types::ByteMessage, util::modes::Modes};

pub struct NameMsg {
    pub name: String
}

impl ByteMessage for NameMsg {
    fn serialize(&self) -> Vec<u8> {
        return Modes::Name.get_send(&self.name.as_bytes().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();
        trace!("Parsing name...");

        let name = String::from_utf8(data)?;

        if name.len() > 20 || name.len() <= 3 {
            trace!("Name is too long / too short {}", name.len());
            return Err(anyhow!("Name packet is too short / too long."));
        }

        return Ok(NameMsg {
            name
        });
    }
}