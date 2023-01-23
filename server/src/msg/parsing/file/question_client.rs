use std::collections::VecDeque;
use log::{debug, trace};
use uuid::Uuid;

use crate::{utils::{converter::{uuid_to_decque, str_to_decque}, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}, msg::parsing::types::WSMessage};
pub struct FileQuestionClientMsg {
    pub filename: String,
    pub sender: Uuid,
    pub receiver: Uuid
}

impl WSMessage for FileQuestionClientMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_filename = str_to_decque(&self.filename);

        let mut b_receiver = uuid_to_decque(&self.receiver);
        let mut b_sender = uuid_to_decque(&self.sender);

        merged.append(&mut b_receiver);
        merged.append(&mut b_sender);
        merged.append(&mut b_filename);

        return Modes::SendFileQuestion.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();
        trace!("Parsing FileQuestionClientMsg length {}", data.len());

        trace!("Receiver");
        let receiver = uuid_from_vec(&mut data)?;
        debug!("Sender");
        let sender = uuid_from_vec(&mut data)?;
        trace!("Filename");
        let filename = String::from_utf8(data)?;

        return Ok(FileQuestionClientMsg {
            filename,
            sender,
            receiver
        });
    }
}