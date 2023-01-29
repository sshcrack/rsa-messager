use colored::Colorize;
use packets::{
    file:: processing::{ready::ChunkReadyMsg, abort::ChunkAbortMsg},
    types::ByteMessage,
};
use tokio_tungstenite::tungstenite::Message;
use log::trace;

use crate::util::{consts::FILE_DOWNLOADS, msg::send_msg};
pub async fn on_chunk_ready(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = ChunkReadyMsg::deserialize(data)?;
    trace!("Received {:?}", msg);

    let state = FILE_DOWNLOADS.read().await;
    let downloader = state.get(&msg.uuid);

    if downloader.is_none() {
        send_msg(Message::binary(ChunkAbortMsg {
            uuid: msg.uuid.clone()
        }.serialize())).await?;

        eprintln!("{}", format!("Could not download chunk of file {}", msg.uuid).on_red());
        return Ok(());
    }

    trace!("Received ready msg. Starting to download with index {}", msg.chunk_index);
    let downloader = downloader.unwrap();
    downloader.start_downloading(msg.chunk_index).await?;

    drop(state);

    Ok(())
}
