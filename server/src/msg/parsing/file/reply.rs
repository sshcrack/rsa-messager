use std::collections::VecDeque;

use log::debug;
use uuid::Uuid;

use crate::{msg::parsing::types::WSMessage, utils::{converter::{uuid_to_decque, str_to_decque, pop_front_vec}, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}};

#[derive(Debug)]
pub struct FileQuestionReplyMsg {
    pub filename: String,
    pub uuid: Uuid,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub accepted: bool
}

impl WSMessage for FileQuestionReplyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_filename = str_to_decque(&self.filename);

        let mut b_uuid = uuid_to_decque(&self.uuid);
        let mut b_receiver = uuid_to_decque(&self.receiver);
        let mut b_sender = uuid_to_decque(&self.sender);
        let b_accepted: u8 = if self.accepted == true { 0 } else { 1 };

        merged.append(&mut b_uuid);
        merged.append(&mut b_receiver);
        merged.append(&mut b_sender);
        merged.push_back(b_accepted);
        merged.append(&mut b_filename);

        return Modes::SendFileQuestionReply.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        debug!("Parsing FileQuestionReply...");
        let mut data = data.clone();

        debug!("Getting uuid...");
        let uuid = uuid_from_vec(&mut data)?;
        debug!("Getting receiver...");
        let receiver = uuid_from_vec(&mut data)?;
        debug!("Getting sender...");
        let sender = uuid_from_vec(&mut data)?;
        debug!("Getting accepted...");
        let accepted = pop_front_vec(&mut data)?;

        debug!("Parsing filename...");
        let filename = String::from_utf8(data)?;

        let accepted = accepted == 1;
        let msg =FileQuestionReplyMsg {
            filename,
            uuid,
            sender,
            receiver,
            accepted
        };

        debug!("Parsed msg {:#?}", msg);
        return Ok(msg);
    }
}