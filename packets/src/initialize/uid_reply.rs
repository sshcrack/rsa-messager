use std::collections::VecDeque;

use uuid::Uuid;

use crate::{types::WSMessage, util::{converter::uuid_to_decque, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}};

pub struct UidReplyMsg {
    pub id: Uuid
}

impl WSMessage for UidReplyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_id = uuid_to_decque(&self.id);
        merged.append(&mut b_id);

        return Modes::UidReply.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let id = uuid_from_vec(&mut data)?;
        return Ok(UidReplyMsg {
            id
        });
    }
}