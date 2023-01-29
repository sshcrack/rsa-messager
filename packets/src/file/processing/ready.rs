use log::trace;
use uuid::Uuid;

use crate::{types::ByteMessage, util::{modes::Modes, tools::{uuid_from_vec, u64_from_vec}}};

#[derive(Debug)]
pub struct ChunkReadyMsg {
    pub uuid: Uuid,
    pub chunk_index: u64
}

impl ByteMessage for ChunkReadyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged = Vec::new();
        let mut b_chunk_index = self.chunk_index.to_le_bytes().to_vec();

        merged.append(&mut self.uuid.as_bytes().to_vec());
        merged.append(&mut b_chunk_index);

        trace!("serialize msg {}", hex::encode(&merged));
        return Modes::SendFileChunkReady.get_send(&merged);
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> where Self: Sized {
        trace!("Parsing ready msg...");
        trace!("Deserialize msg {}", hex::encode(data));
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        let chunk_index = u64_from_vec(&mut data)?;

        trace!("Done.");
        return Ok(ChunkReadyMsg {
            uuid,
            chunk_index
        })
    }
}