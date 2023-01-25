use std::collections::VecDeque;

use uuid::Uuid;

use crate::{types::ByteMessage, util::{converter::uuid_to_decque, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}};

pub struct UidReplyMsg {
    pub uuid: Uuid
}

impl ByteMessage for UidReplyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();
        let mut b_uuid = uuid_to_decque(&self.uuid);
        merged.append(&mut b_uuid);

        return Modes::UidReply.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let uuid = uuid_from_vec(&mut data)?;
        return Ok(UidReplyMsg {
            uuid
        });
    }
}