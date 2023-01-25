use std::{collections::VecDeque, ops::Range};

use anyhow::anyhow;


pub fn vec_to_decque<T>(v: Vec<T>) -> VecDeque<T> {
    let mut r: VecDeque<T> = VecDeque::new();
    for i in v {
        r.push_back(i);
    }

    return r;
}

pub fn decque_to_vec<T>(v: VecDeque<T>) -> Vec<T> {
    let mut r: Vec<T> = Vec::new();
    for i in v {
        r.push(i);
    }

    return r;
}

pub fn extract_vec(range: Range<usize>, v: &mut Vec<u8>) -> anyhow::Result<Vec<u8>> {
    if range.start >= v.len() || range.end >= v.len() {
        return Err(anyhow!(format!("Could not extract vec with range {:?} as vec is {} long", range, v.len())));
    }

    let splice = v.splice(range, vec![]);
    let out: Vec<u8> = splice.collect();

    return Ok(out);
}