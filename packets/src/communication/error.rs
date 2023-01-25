use crate::{types::ByteMessage, util::modes::Modes};

pub struct ErrorMsg {
    pub error: String
}

impl ByteMessage for ErrorMsg {
    fn serialize(&self) -> Vec<u8> {
        return Modes::Error.get_send(&self.error.as_bytes().to_vec());
    }

    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> {
        let data = data.clone();

        let error = String::from_utf8(data)?;
        return Ok(ErrorMsg {
            error
        });
    }
}