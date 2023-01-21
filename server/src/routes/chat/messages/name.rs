use uuid::Uuid;

use crate::utils::types::Users;

pub async fn on_name(msg: &Vec<u8>, curr_id: &Uuid, users: &Users) {
    let msg = msg.to_vec();
    let msg = String::from_utf8(msg);

    if msg.is_err() {
        eprintln!("Invalid name packet.");
        return;
    }

    let msg = msg.unwrap();
    if msg.len() > 20 || msg.len() <= 3 {
        println!("Can't change, name too long / too short.");
        return;
    }

    let mut state = users.write().await;
    let info = state.get_mut(&curr_id);

    if info.is_some() {
        let i = info.unwrap();
        i.name = Some(msg);
    }

    drop(state);
    println!("Name set.");
}
