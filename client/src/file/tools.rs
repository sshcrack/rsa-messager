use anyhow::anyhow;
use packets::file::types::FileInfo;
use uuid::Uuid;

use crate::util::consts::PENDING_FILES;

pub async fn get_pending_file(uuid: Uuid) -> anyhow::Result<FileInfo> {
    let state = PENDING_FILES.read().await;
    let temp = state.get(&uuid);
    if temp.is_none() {
        return Err(anyhow!("Could not find pending upload"));
    }

    return Ok(temp.unwrap().to_owned());
}

