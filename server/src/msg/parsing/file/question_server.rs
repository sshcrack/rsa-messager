use std::collections::VecDeque;

use log::{debug, trace};
use uuid::Uuid;

use crate::{utils::{converter::{uuid_to_decque, str_to_decque}, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}, msg::parsing::types::WSMessage};

#[derive(Debug)]
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
        trace!("Parsing FileQuestionServerMsg len {}", data.len());

        trace!("Parsing uuid...");
        let uuid = uuid_from_vec(&mut data)?;
        trace!("Parsing receiver...");
        let receiver = uuid_from_vec(&mut data)?;
        trace!("Parsing sender...");
        let sender = uuid_from_vec(&mut data)?;
        trace!("Parsing filename...");
        let filename = String::from_utf8(data)?;

        let msg = FileQuestionServerMsg {
            filename,
            uuid,
            sender,
            receiver
        };

        debug!("Parsed FileQuestionServerMsg {:?}", msg);
        return Ok(msg);
    }
}