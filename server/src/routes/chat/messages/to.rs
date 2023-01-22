use std::collections::VecDeque;

use warp::ws::Message;

use crate::utils::{types::Users, tools::{uuid_from_vec, vec_to_decque, decque_to_vec, send_msg_specific}, modes::Modes};


pub async fn on_to(msg: Vec<u8>, users: &Users) -> anyhow::Result<()> {
    let mut msg = msg.to_vec();

    let send_to = uuid_from_vec(&mut msg)?;

    let mut merged = VecDeque::new();

    let mut uuid_bytes = vec_to_decque(send_to.as_bytes().to_vec());
    let mut msg_copy = vec_to_decque(msg);

    merged.append(&mut uuid_bytes);
    merged.append(&mut msg_copy);

    let merged = decque_to_vec(merged);
    let packet = Modes::From.get_send(&merged);

    send_msg_specific(send_to, users, Message::binary(packet)).await?;

    Ok(())
}
