use std::collections::VecDeque;

use log::trace;
use uuid::Uuid;

use crate::{types::ByteMessage, util::{converter::{pop_front_vec, uuid_to_decque}, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}};

#[derive(Debug, Clone)]
pub struct FileQuestionReplyMsg {
    pub uuid: Uuid,
    pub accepted: bool
}

impl ByteMessage for FileQuestionReplyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();

        let mut b_uuid = uuid_to_decque(&self.uuid);
        let b_accepted: u8 = if self.accepted == true { 1 } else { 0 };

        merged.append(&mut b_uuid);
        merged.push_back(b_accepted);

        return Modes::SendFileQuestionReply.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        let accepted = pop_front_vec(&mut data)?;
        let accepted = accepted == 1;

        let msg = FileQuestionReplyMsg {
            uuid,
            accepted
        };

        trace!("FileQuestionReplyMsg parsed: {:#?}", msg);
        return Ok(msg);
    }
}