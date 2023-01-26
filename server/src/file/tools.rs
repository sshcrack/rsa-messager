use std::path::PathBuf;

use anyhow::anyhow;
use packets::file::types::FileInfo;
use tokio::fs::create_dir;
use uuid::Uuid;

use super::consts::{PENDING_UPLOADS, CHUNK_DIR, UPLOADING_FILES};

pub async fn get_pending_file(uuid: &Uuid) -> anyhow::Result<FileInfo> {
    let state = PENDING_UPLOADS.read().await;
    let info = state.get(&uuid);
    let out = if info.is_none() { Err(anyhow!("Could not find upload")) } else { Ok(info.unwrap().clone()) };

    drop(state);
    return out;
}


pub async fn get_uploading_file(uuid: &Uuid) -> anyhow::Result<FileInfo> {
    let state = UPLOADING_FILES.read().await;
    let info = state.get(&uuid);
    let out = if info.is_none() { Err(anyhow!("Could not find upload on uploading files")) } else { Ok(info.unwrap().clone()) };

    drop(state);
    return out;
}


pub async fn get_chunk_file(uuid: &Uuid, chunk_index: u64) -> anyhow::Result<PathBuf> {
    let mut file = CHUNK_DIR.to_path_buf().clone();
    if !CHUNK_DIR.is_dir() { create_dir(file.clone()).await?; }

    file.push(format!("{}-{}.bin", uuid.to_string(), chunk_index));
    return Ok(file);
}