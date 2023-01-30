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

    let state = FILE_UPLOADS.read().await;
    let uploader = state.get(&msg.uuid);

    if uploader.is_none() {
        send_msg(Message::binary(ChunkAbortMsg {
            uuid: msg.uuid.clone()
        }.serialize())).await?;

        eprintln!("{}", format!("Could not download chunk of file {} (uploader is none)", msg.uuid).on_red());
        return Ok(());
    }

    let uploader = uploader.unwrap();
    let max_chunks = uploader.get_max_chunks();
    let new_chunk = msg.chunk_index +1;

    if new_chunk >= max_chunks {
        let FileInfo {filename, receiver, ..} = uploader.get_file_info();
        let receiver_name = uuid_to_name(receiver).await?;

        println!("{}", format!("File '{}' was successfully sent to {}.", filename.yellow(), receiver_name.blue().bold()).green());
        return Ok(());
    }

    uploader.start_upload(new_chunk).await?;
    drop(state);

    Ok(())
}
