use colored::Colorize;
use packets::{
    file:: processing::{abort::ChunkAbortMsg, downloaded::ChunkDownloadedMsg},
    types::ByteMessage,
};
use log::trace;
use tokio_tungstenite::tungstenite::Message;
use crate::util::{consts::FILE_UPLOADS, msg::send_msg};

pub async fn on_chunk_downloaded(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = ChunkDownloadedMsg::deserialize(data)?;
    trace!("Received {:?}", msg);

    let state = FILE_UPLOADS.read().await;
    let uploader = state.get(&msg.uuid);

    if uploader.is_none() {
        send_msg(Message::binary(ChunkAbortMsg {
            uuid: msg.uuid.clone()
        }.serialize())).await?;

        eprintln!("{}", format!("Could not download chunk of file {} (uploader is none)", msg.uuid).on_red());
        return Ok(());
    }

    let downloader = uploader.unwrap();
    downloader.start_upload(msg.chunk_index).await?;

    drop(state);

    Ok(())
}
