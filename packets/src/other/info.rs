use openssl::{pkey::Public, rsa::Rsa};

use crate::{util::{tools::usize_from_vec, vec::extract_vec}};


#[derive(Debug, Clone)]
pub struct UserInfoBasic {
    pub name: Option<String>,
    pub public_key: Option<Rsa<Public>>,
}

impl UserInfoBasic {
    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut merged = Vec::new();
        let name = self.name.clone().unwrap_or("".to_owned());
        let name_len = name.len();

        let mut b_name_len = name_len.to_le_bytes().to_vec();
        let mut b_name = name.as_bytes().to_vec();

        let mut pubkey = Vec::new();
        if self.public_key.is_some() {
            let key = self.public_key.clone().unwrap();
            pubkey = key.public_key_to_pem()?
        }

        let mut pubkey_len = pubkey.len().to_le_bytes().to_vec();

        merged.append(&mut b_name_len);
        merged.append(&mut b_name);
        merged.append(&mut pubkey_len);
        merged.append(&mut pubkey);

        return Ok(merged);
    }

    pub fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> where Self: Sized {
        let mut data = data.clone();
        let name_len = usize_from_vec(&mut data)?;

        let b_name = extract_vec(0..name_len, &mut data)?;
        let name = String::from_utf8(b_name)?;

        let mut name_out = None;
        if name_len != 0 {
            name_out = Some(name);
        }

        let pubkey_len = usize_from_vec(&mut data)?;
        let pubkey_pem = extract_vec(0..pubkey_len, &mut data)?;

        let mut public_key= None;
        if pubkey_len != 0 {
            public_key = Some(Rsa::public_key_from_pem(&pubkey_pem)?);
        }

        return Ok(Self {
            name: name_out,
            public_key
        });
    }
}