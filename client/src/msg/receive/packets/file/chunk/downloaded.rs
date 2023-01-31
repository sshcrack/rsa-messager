use colored::Colorize;
use packets::{
    file::{ processing::{abort::ChunkAbortMsg, downloaded::ChunkDownloadedMsg}, types::FileInfo},
    types::ByteMessage,
};
use log::trace;
use tokio_tungstenite::tungstenite::Message;
use crate::util::{consts::FILE_UPLOADS, msg::send_msg, tools::uuid_to_name};

pub async fn on_chunk_downloaded(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let msg = ChunkDownloadedMsg::deserialize(data)?;
    trace!("Received {:?}", msg);

    trace!("Waiting for file uploads");
    let state = FILE_UPLOADS.read().await;
    let uploader = state.get(&msg.uuid);

    trace!("Got uploader.");
    if uploader.is_none() {
        send_msg(Message::binary(ChunkAbortMsg {
            uuid: msg.uuid.clone()
        }.serialize())).await?;

        eprintln!("{}", format!("Could not download chunk of file {} (uploader is none)", msg.uuid).on_red());
        return Ok(());
    }

    let uploader = uploader.unwrap();
    trace!("Getting chunks completed");
    let completed = uploader.get_chunks_completed().await;
    let max_chunks = uploader.get_max_chunks();


    if completed >= max_chunks {
        let FileInfo {filename, receiver, ..} = uploader.get_file_info();
        let receiver_name = uuid_to_name(receiver).await?;

        println!("{}", format!("File '{}' was successfully sent to {}.", filename.yellow(), receiver_name.blue().bold()).green());
        return Ok(());
    }

    let max_chunks_size = usize::try_from(max_chunks)?;
    trace!("getting chunks processing");
    let processing = uploader.get_chunks_processing().await;
    let chunks_left = max_chunks_size - processing.len();

    trace!("ChunksLeft: {} total size {}", chunks_left, max_chunks_size);
    if chunks_left <= 0 { return Ok(()) }

    trace!("On next uploader");
    uploader.on_next().await?;
    trace!("Done on next");
    drop(state);

    Ok(())
}
