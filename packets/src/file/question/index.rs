use std::collections::VecDeque;

use log::trace;
use uuid::Uuid;

use crate::{
    types::WSMessage,
    util::{
        converter::{str_to_decque, uuid_to_decque},
        modes::Modes,
        tools::{u64_from_vec, uuid_from_vec},
        vec::{decque_to_vec, vec_to_decque},
    },
};

#[derive(Debug)]
pub struct FileQuestionMsg {
    pub filename: String,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub uuid: Uuid,
    pub size: u64,
    pub secret: u64,
}

impl WSMessage for FileQuestionMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_filename = str_to_decque(&self.filename);

        let mut b_uuid = uuid_to_decque(&self.uuid);
        let mut b_receiver = uuid_to_decque(&self.receiver);
        let mut b_sender = uuid_to_decque(&self.sender);
        let mut b_size = vec_to_decque(self.size.to_le_bytes().to_vec());
        let mut b_secret = vec_to_decque(self.secret.to_le_bytes().to_vec());

        merged.append(&mut b_uuid);
        merged.append(&mut b_receiver);
        merged.append(&mut b_sender);
        merged.append(&mut b_size);
        merged.append(&mut b_secret);
        merged.append(&mut b_filename);

        return Modes::SendFileQuestion.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        trace!("Parsing file question...");
        let mut data = data.clone();

        trace!("Parsing uuid...");
        let uuid = uuid_from_vec(&mut data)?;

        trace!("Parsing receiver...");
        let receiver = uuid_from_vec(&mut data)?;

        trace!("Parsing sender...");
        let sender = uuid_from_vec(&mut data)?;

        trace!("Parsing size...");
        let size = u64_from_vec(&mut data)?;

        trace!("Parsing secret...");
        let secret = u64_from_vec(&mut data)?;

        trace!("Parsing filename...");
        let filename = String::from_utf8(data)?;

        let msg = FileQuestionMsg {
            filename,
            uuid,
            sender,
            receiver,
            size,
            secret,
        };

        trace!("FileQuestion info is {:?}", msg);
        return Ok(msg);
    }
}
