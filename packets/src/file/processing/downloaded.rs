use uuid::Uuid;

use crate::{types::ByteMessage, util::{modes::Modes, tools::{uuid_from_vec, u64_from_vec}}};

#[derive(Debug)]
pub struct ChunkDownloadedMsg {
    pub uuid: Uuid,
    pub chunk_index: u64
}

impl ByteMessage for ChunkDownloadedMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged = Vec::new();
        let mut b_chunk_index = self.chunk_index.to_le_bytes().to_vec();

        merged.append(&mut self.uuid.as_bytes().to_vec());
        merged.append(&mut b_chunk_index);

        return Modes::SendFileChunkDownloaded.get_send(&merged);
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> where Self: Sized {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        let chunk_index = u64_from_vec(&mut data)?;

        return Ok(ChunkDownloadedMsg {
            uuid,
            chunk_index
        })
    }
}