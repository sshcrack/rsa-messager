use std::collections::VecDeque;

use uuid::Uuid;

use crate::{types::ByteMessage, util::{converter::uuid_to_decque, vec::{decque_to_vec, vec_to_decque}, modes::Modes, tools::uuid_from_vec}};

pub struct ToMsg {
    pub msg: Vec<u8>,
    pub receiver: Uuid,
}

impl ByteMessage for ToMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();

        let mut b_msg = vec_to_decque(self.msg.clone());
        let mut b_receiver = uuid_to_decque(&self.receiver);

        merged.append(&mut b_receiver);
        merged.append(&mut b_msg);

        return Modes::To.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let receiver = uuid_from_vec(&mut data)?;

        return Ok(ToMsg {
            msg: data,
            receiver
        });
    }
}