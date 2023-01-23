use std::collections::VecDeque;

use uuid::Uuid;

use crate::{types::WSMessage, util::{converter::{str_to_decque, uuid_to_decque}, vec::{decque_to_vec, vec_to_decque}, modes::Modes, tools::{uuid_from_vec, u64_from_vec}}};

pub struct FileQuestionMsg {
    pub filename: String,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub uuid: Uuid,
    pub size: u64
}

impl WSMessage for FileQuestionMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_filename = str_to_decque(&self.filename);

        let mut b_uuid = uuid_to_decque(&self.uuid);
        let mut b_receiver = uuid_to_decque(&self.receiver);
        let mut b_sender = uuid_to_decque(&self.sender);
        let mut b_size = vec_to_decque(self.size.to_le_bytes().to_vec());

        merged.append(&mut b_uuid);
        merged.append(&mut b_receiver);
        merged.append(&mut b_sender);
        merged.append(&mut b_size);
        merged.append(&mut b_filename);

        return Modes::SendFileQuestion.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        let receiver = uuid_from_vec(&mut data)?;
        let sender = uuid_from_vec(&mut data)?;
        let size = u64_from_vec(&mut data)?;
        let filename = String::from_utf8(data)?;

        return Ok(FileQuestionMsg {
            filename,
            uuid,
            sender,
            receiver,
            size
        });
    }
}