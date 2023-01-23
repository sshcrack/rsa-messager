use std::collections::VecDeque;

use uuid::Uuid;

use crate::{msg::parsing::types::WSMessage, util::{converter::{str_to_decque, uuid_to_decque}, modes::Modes, vec::decque_to_vec, tools::uuid_from_vec}};

pub struct FileQuestionServerMsg {
    pub filename: String,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub uuid: Uuid
}

impl WSMessage for FileQuestionServerMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_filename = str_to_decque(&self.filename);

        let mut b_uuid = uuid_to_decque(&self.uuid);
        let mut b_receiver = uuid_to_decque(&self.receiver);
        let mut b_sender = uuid_to_decque(&self.sender);

        merged.append(&mut b_uuid);
        merged.append(&mut b_receiver);
        merged.append(&mut b_sender);
        merged.append(&mut b_filename);

        return Modes::SendFileQuestion.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        let receiver = uuid_from_vec(&mut data)?;
        let sender = uuid_from_vec(&mut data)?;
        let filename = String::from_utf8(data)?;

        return Ok(FileQuestionServerMsg {
            filename,
            uuid,
            sender,
            receiver
        });
    }
}