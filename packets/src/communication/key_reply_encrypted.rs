use uuid::Uuid;

use crate::{util::{converter::uuid_to_vec, modes::Modes, tools::uuid_from_vec}, types::ByteMessage};

pub struct EncryptedSymmKeyReplyMsg {
    pub key: Vec<u8>,
    pub user: Uuid,
}

impl ByteMessage for EncryptedSymmKeyReplyMsg {
    fn serialize(&self) -> Vec<u8> {
        let mut merged: Vec<u8> = Vec::new();
        let mut b_user = uuid_to_vec(&self.user);
        let mut b_key = self.key.clone();

        merged.append(&mut b_user);
        merged.append(&mut b_key);

        return Modes::SymmKey.get_send(&merged);
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let user = uuid_from_vec(&mut data)?;
        return Ok(Self {
            key: data,
            user
        });
    }
}