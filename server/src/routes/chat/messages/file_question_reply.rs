use std::collections::VecDeque;

use anyhow::anyhow;
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{
    tools::{send_msg_specific, uuid_from_vec, vec_to_decque, decque_to_vec},
    types::Users, modes::Modes,
};

pub async fn on_file_question_reply(
    data: &Vec<u8>,
    curr_id: &Uuid,
    users: &Users
) -> anyhow::Result<()> {
    let mut data = data.to_vec();

    let receiver = uuid_from_vec(&mut data)?;
    let mut data = vec_to_decque(data);

    let accepted = data.pop_front();

    let data = decque_to_vec(data);

    if accepted.is_none() {
        eprintln!("Invalid return packet");
        return Err(anyhow!("Invalid reply packet"));
    }

    let accepted = accepted.unwrap();

    // Doing this to verify filename is actually string
    let filename = String::from_utf8(data)?;

    let mut merged = VecDeque::new();
    let mut b_curr_id = vec_to_decque(curr_id.as_bytes().to_vec());
    let mut b_filename = vec_to_decque(filename.as_bytes().to_vec());

    merged.append(&mut b_curr_id);
    merged.push_back(accepted);
    merged.append(&mut b_filename);
    let merged = decque_to_vec(merged);

    println!("Sending {} to {}", filename, receiver);
    let to_send = Modes::SendFileQuestionReply.get_send(&merged);
    send_msg_specific(receiver, users, Message::binary(to_send)).await?;
    Ok(())
}
