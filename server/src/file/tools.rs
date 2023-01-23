use anyhow::anyhow;
use packets::file::types::FileInfo;
use uuid::Uuid;

use super::consts::PENDING_UPLOADS;

pub async fn get_pending_file(uuid: Uuid) -> anyhow::Result<FileInfo> {
    let state = PENDING_UPLOADS.read().await;
    let info = state.get(&uuid);
    let out = if info.is_none() { Err(anyhow!("Could not find upload")) } else { Ok(info.unwrap().clone()) };

    drop(state);
    return out;
}