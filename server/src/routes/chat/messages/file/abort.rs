use std::cmp::Ordering;

use anyhow::anyhow;
use log::trace;
use packets::{file::processing::abort::ChunkAbortMsg, types::ByteMessage};
use tokio::fs::{read_dir, remove_file};
use uuid::Uuid;
use warp::ws::Message;

use crate::file::{tools::{get_uploading_file, get_pending_file}, consts::{CHUNK_DIR, PENDING_UPLOADS, UPLOADING_FILES}};

pub async fn on_chunk_abort(data: &Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let msg = ChunkAbortMsg::deserialize(data)?;
    trace!("ChunkAbort: {:?}", msg);

    let file = get_pending_file(&msg.uuid).await.or(get_uploading_file(&msg.uuid).await)?;
    if my_id.cmp(&file.receiver) != Ordering::Equal && my_id.cmp(&file.sender) != Ordering::Equal {
        trace!("Cannot abort upload task if client is not the sender or receiver of it.");
        return Err(anyhow!("Invalid receiver / sender"));
    }

    let chunk_dir = CHUNK_DIR.as_path();
    let mut files = read_dir(chunk_dir).await?;

    while let Ok(file) = files.next_entry().await {
        if file.is_none() {
            continue;
        }

        let file = file.unwrap();
        let name = file.file_name();
        let name = name.to_str();
        if name.is_none() {
            continue;
        }

        let name = name.unwrap().to_string();
        let is_chunk_file = name.starts_with(&msg.uuid.to_string());

        if is_chunk_file {
            trace!("Aborting. Removing file {} in chunks.", name);
            remove_file(file.path()).await?;
        }
    }

    trace!("Removing pending uploads and files...");
    PENDING_UPLOADS.write().await.remove(&msg.uuid);
    UPLOADING_FILES.write().await.remove(&msg.uuid);

    return Ok(())
}
