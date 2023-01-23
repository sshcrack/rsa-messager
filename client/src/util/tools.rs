use anyhow::anyhow;
use uuid::Uuid;

use crate::web::user_info::get_user_info;

use super::consts::UUID_SIZE;


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


pub async fn uuid_to_name(uuid: Uuid) -> anyhow::Result<String> {
    let info = get_user_info(&uuid).await?;

    if info.name.is_some() {
        return Ok(info.name.unwrap());
    }

    return Ok(uuid.to_string());
}
