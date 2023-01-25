use log::debug;
use packets::{initialize::name::NameMsg, types::ByteMessage};
use uuid::Uuid;

use crate::file::consts::USERS;

pub async fn on_name(data: &Vec<u8>, curr_id: &Uuid) -> anyhow::Result<()>{
    let NameMsg { name } = NameMsg::deserialize(data)?;

    let mut state = USERS.write().await;
    let info = state.get_mut(&curr_id);

    if info.is_some() {
        let i = info.unwrap();
        i.name = Some(name.clone());
    }

    drop(state);
    debug!("Name set. ({name})");
    Ok(())
}
