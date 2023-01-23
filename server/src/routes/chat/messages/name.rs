use anyhow::anyhow;
use log::debug;
use uuid::Uuid;

use crate::utils::types::Users;

pub async fn on_name(msg: &Vec<u8>, curr_id: &Uuid, users: &Users) -> anyhow::Result<()>{
    let msg = msg.to_vec();
    let msg = String::from_utf8(msg)?;

    if msg.len() > 20 || msg.len() <= 3 {
        return Err(anyhow!("Can't change, name too long / too short."));
    }

    let mut state = users.write().await;
    let info = state.get_mut(&curr_id);

    if info.is_some() {
        let i = info.unwrap();
        i.name = Some(msg.clone());
    }

    drop(state);
    debug!("Name set. ({msg})");
    Ok(())
}
