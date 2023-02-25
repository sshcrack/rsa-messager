use std::collections::VecDeque;

use anyhow::anyhow;
use uuid::Uuid;

use super::vec::{vec_to_decque, decque_to_vec};

pub fn str_to_decque(s: &str) -> VecDeque<u8> {
    return vec_to_decque(s.as_bytes().to_vec());
}

pub fn uuid_to_decque(uuid: &Uuid) -> VecDeque<u8> {
    return vec_to_decque(uuid.as_bytes().to_vec());
}

pub fn uuid_to_vec(uuid: &Uuid) -> Vec<u8> {
    return uuid.as_bytes().to_vec();
}


pub fn pop_front_vec(vec: &mut Vec<u8>) -> anyhow::Result<u8> {
    let mut vec_d = vec_to_decque(vec.clone());
    let res = vec_d.pop_front();
    if res.is_none() {
        return Err(anyhow!("Could not pop front, is nothing."));
    }

    let res = res.unwrap();
    *vec = decque_to_vec(vec_d);

    return Ok(res);
}