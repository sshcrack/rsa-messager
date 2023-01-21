use std::collections::VecDeque;

use warp::ws::Message;

use crate::utils::{types::Users, tools::{uuid_from_vec, vec_to_decque, decque_to_vec}, modes::Modes};


pub async fn on_to(msg: Vec<u8>, users: &Users) {
    let mut msg = msg.to_vec();

    let send_to = uuid_from_vec(&mut msg);
    if send_to.is_err() {
        eprintln!("Invalid format: {}", send_to.unwrap_err());
        return;
    }

    let send_to = send_to.unwrap();
    println!("Uuid {} obtained.", send_to);
    let mut merged = VecDeque::new();

    let mut uuid_bytes = vec_to_decque(send_to.as_bytes().to_vec());
    let mut msg_copy = vec_to_decque(msg);

    merged.append(&mut uuid_bytes);
    merged.append(&mut msg_copy);

    let merged = decque_to_vec(merged);
    let packet = Modes::From.get_send(&merged);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, info) in users.read().await.iter() {
        if send_to.to_string().eq(&uid.to_string()) {
            let tx = &info.sender;

            if let Err(_disconnected) = tx.send(Message::binary(packet.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}
