use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::ws::Message;

use crate::{
    utils::types::Users,
};

pub async fn user_message(my_id: Uuid, msg: Message, users: &Users, tx: &UnboundedSender<Message>) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    if msg.eq("getuid") {
        let res = tx.send(Message::text(format!("UID:{}", my_id)));
        if res.is_err() {
            eprintln!("{}", res.unwrap_err());
        }
    }

    if msg.starts_with("to:") {
        handle_msg_to(my_id, msg, users).await;
        return;
    }

    if msg.starts_with("setpubkey:") {
        // TODO if i want to, check size of key here
        let msg = msg.replace("setpubkey:", "");
        let mut state = users.write().await;
        let info = state.get_mut(&my_id);

        if info.is_some() {
            let i = info.unwrap();
            i.public_key = Some(msg);
        }

        drop(state);
        println!("Pubkey set.");
        return;
    }

    if msg.starts_with("name:") {
        let msg = msg.replace("name:", "");
        if msg.len() > 20 {
            println!("Can't change, name too long.");
            return;
        }

        let mut state = users.write().await;
        let info = state.get_mut(&my_id);

        if info.is_some() {
            let i = info.unwrap();
            i.name = Some(msg);
        }

        drop(state);
        println!("Name set.");
    }
}

pub async fn handle_msg_to(my_id: Uuid, msg: &str, users: &Users) {
    let mut parts: Vec<&str> = msg.split(":").collect();
    let parts_c = parts.clone();
    let send_to = parts_c.get(1);
    if send_to.is_none() {
        println!("Invalid format");
        return;
    }

    let send_to = send_to.unwrap();
    parts.remove(0);
    parts.remove(0);

    let left = parts.join(":");

    let new_msg = format!("from:{}:{}", my_id, left);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, info) in users.read().await.iter() {
        if send_to.to_string().eq(&uid.to_string()) {
            let tx = &info.sender;

            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}
