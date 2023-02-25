use openssl::{rsa::Rsa, pkey::{Private, Public}};
use uuid::Uuid;

use crate::{util::{converter::uuid_to_vec, modes::Modes, tools::uuid_from_vec}, other::key_iv::KeyIVPair};

pub struct SymmKeyReplyMsg {
    pub key: KeyIVPair,
    pub user: Uuid,
}

impl SymmKeyReplyMsg {
    pub fn serialize(&self, rec_key: Rsa<Public>) -> anyhow::Result<Vec<u8>> {
        let mut merged: Vec<u8> = Vec::new();

        let mut b_encrypted_key = self.key.serialize(&rec_key)?;
        let mut b_user = uuid_to_vec(&self.user);

        merged.append(&mut b_user);
        merged.append(&mut b_encrypted_key);

        return Ok(Modes::SymmKey.get_send(&merged));
    }

    pub fn deserialize(data: &Vec<u8>, key: Rsa<Private>) -> anyhow::Result<Self> {
        let mut data = data.clone();

        let user = uuid_from_vec(&mut data)?;

        let decrypted_key = KeyIVPair::deserialize_mut(&mut data, &key)?;
        return Ok(SymmKeyReplyMsg {
            key: decrypted_key,
            user
        });
    }
}