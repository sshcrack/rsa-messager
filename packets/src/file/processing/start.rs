use std::collections::VecDeque;

use log::trace;
use uuid::Uuid;

use crate::{types::ByteMessage, util::{modes::Modes, tools::{uuid_from_vec, u64_from_vec}, converter::uuid_to_decque, vec::{vec_to_decque, decque_to_vec}}};

pub struct FileStartProcessing {
    pub uuid: Uuid,
    pub threads: u64
}

impl ByteMessage for FileStartProcessing {
    fn serialize(&self) -> Vec<u8> {
        let mut merged = VecDeque::new();
        let mut b_uuid = uuid_to_decque(&self.uuid);
        let mut b_threads = vec_to_decque(self.threads.to_le_bytes().to_vec());

        merged.append(&mut b_uuid);
        merged.append(&mut b_threads);

        return Modes::SendFileStartProcessing.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        trace!("Parsing FileStartProcessing UUID...");
        let uuid = uuid_from_vec(&mut data)?;

        trace!("Parsing FileStartProcessing Threads...");
        let threads = u64_from_vec(&mut data)?;
        return Ok(FileStartProcessing {
            uuid,
            threads
        });
    }
}