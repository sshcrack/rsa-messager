use log::debug;
use packets::{initialize::name::NameMsg, types::WSMessage};
use uuid::Uuid;

use crate::utils::types::Users;

pub async fn on_name(data: &Vec<u8>, curr_id: &Uuid, users: &Users) -> anyhow::Result<()>{
    let NameMsg { name } = NameMsg::deserialize(data)?;

    let mut state = users.write().await;
    let info = state.get_mut(&curr_id);

    if info.is_some() {
        let i = info.unwrap();
        i.name = Some(name.clone());
    }

    drop(state);
    debug!("Name set. ({name})");
    Ok(())
}
