use anyhow::anyhow;
use uuid::Uuid;
use warp::ws::Message;

use super::types::{TXChannel, Users};

pub const UUID_SIZE: usize = 16;
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

pub fn send_msg(tx: &TXChannel, msg: Message) -> anyhow::Result<()> {
    let e = tx.send(msg);

    if e.is_err() {
        eprint!("{}", e.unwrap_err());
        return Err(anyhow!("TX Send error."));
    }

    Ok(())
}

pub async fn send_msg_specific(id: Uuid, users: &Users, msg: Message) -> anyhow::Result<()> {
    let mut found = false;

    for (&uid, info) in users.read().await.iter() {
        if id.to_string().eq(&uid.to_string()) {
            let tx = &info.sender;

            found = true;
            if let Err(_disconnected) = tx.send(msg.clone()) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }

    if !found {
        return Err(anyhow!(format!("Could not send to {}. User not in list.", id)));
    }

    return Ok(());
}