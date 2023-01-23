use anyhow::anyhow;
use uuid::Uuid;
use warp::ws::Message;

use super::types::{TXChannel, Users};

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