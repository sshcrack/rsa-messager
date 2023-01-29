use uuid::Uuid;

use crate::{types::ByteMessage, util::{modes::Modes, tools::uuid_from_vec}};

#[derive(Debug)]
pub struct ChunkAbortMsg {
    pub uuid: Uuid
}

impl ByteMessage for ChunkAbortMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged = Vec::new();
        merged.append(&mut self.uuid.as_bytes().to_vec());

        return Modes::SendFileAbort.get_send(&merged);
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> where Self: Sized {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        return Ok(ChunkAbortMsg {
            uuid
        })
    }
}