use anyhow::anyhow;
use uuid::Uuid;

use crate::consts::{U64_SIZE, UUID_SIZE};

pub fn bytes_to_uuid(v: &Vec<u8>) -> anyhow::Result<Uuid> {
    if v.len() != UUID_SIZE {
        return Err(anyhow!(format!(
            "Invalid length of uuid. Length is {}",
            v.len()
        )));
    }

    let mut buff: [u8; UUID_SIZE] = [0; UUID_SIZE];
    for i in 0..UUID_SIZE {
        buff[i] = v.get(i).unwrap().to_owned();
    }

    let uuid = Uuid::from_bytes(buff);
    return Ok(uuid);
}

pub fn uuid_from_vec(v: &mut Vec<u8>) -> anyhow::Result<Uuid> {
    if v.len() < UUID_SIZE {
        return Err(anyhow!("Could not get uuid from packet."));
    }

    let uuid = v.splice(0..UUID_SIZE, vec![]);
    let uuid = Vec::from_iter(uuid);

    let uuid = bytes_to_uuid(&uuid)?;
    return Ok(uuid);
}

pub fn u64_from_vec(v: &mut Vec<u8>) -> anyhow::Result<u64> {
    if v.len() < U64_SIZE {
        return Err(anyhow!(format!(
            "Cannot parse u64 as vector is not long enough ({} items)",
            v.len()
        )));
    }

    let temp = v.splice(0..U64_SIZE, vec![]);
    let temp = Vec::from_iter(temp);

    let mut buff: [u8; U64_SIZE] = [0; U64_SIZE];
    for i in 0..U64_SIZE {
        buff[i] = temp.get(i).unwrap().to_owned();
    }

    let numb = u64::from_le_bytes(buff);
    Ok(numb)
}

pub fn usize_to_vec(size: usize) -> anyhow::Result<Vec<u8>> {
    let raw: u64 = u64::try_from(size)?;

    return Ok(raw.to_le_bytes().to_vec());
}

pub fn vec_to_usize(v: &mut Vec<u8>) -> anyhow::Result<usize> {
    let raw_numb = u64_from_vec(v)?;
    let numb = usize::try_from(raw_numb);

    if numb.is_err() {
        eprintln!("Err: {}", numb.unwrap_err());
        return Err(anyhow!("Usize could not be parsed. Number exceeds usize max."));
    }

    Ok(numb.unwrap())
}