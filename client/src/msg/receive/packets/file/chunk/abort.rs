use anyhow::anyhow;
use packets::{
    file:: processing::abort::ChunkAbortMsg,
    types::ByteMessage,
};
use crate::util::consts::{FILE_DOWNLOADS, FILE_UPLOADS};

pub async fn on_chunk_abort(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = ChunkAbortMsg::deserialize(data)?;

    let state = FILE_DOWNLOADS.write().await;
    let downloader = state.get(&msg.uuid);

    if downloader.is_some() {
        downloader.unwrap().abort().await;
        drop(state);
        return Ok(());
    }

    drop(state);

    let state = FILE_UPLOADS.read().await;
    let uploader = state.get(&msg.uuid);

    if uploader.is_some() {
        uploader.unwrap().abort().await;
        drop(state);
        return Ok(());
    }

    Err(anyhow!("Received abort msg but could not find file"))
}
