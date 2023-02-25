use std::collections::VecDeque;

use uuid::Uuid;

use crate::{types::ByteMessage, util::{converter::uuid_to_decque, vec::decque_to_vec, modes::Modes, tools::uuid_from_vec}};

pub struct WantSymmKeyMsg {
    pub user: Uuid,
}

impl ByteMessage for WantSymmKeyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: VecDeque<u8> = VecDeque::new();

        let mut b_user = uuid_to_decque(&self.user);
        merged.append(&mut b_user);

        return Modes::WantSymmKey.get_send(&decque_to_vec(merged));
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let user = uuid_from_vec(&mut data)?;

        return Ok(WantSymmKeyMsg {
            user
        });
    }
}