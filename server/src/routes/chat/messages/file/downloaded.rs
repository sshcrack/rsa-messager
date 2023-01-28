use std::cmp::Ordering;

use log::trace;
use packets::{file::processing::downloaded::ChunkDownloadedMsg, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::{file::tools::get_uploading_file, utils::tools::send_msg_specific};

pub async fn on_chunk_downloaded(data: &Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let msg = ChunkDownloadedMsg::deserialize(data)?;
    trace!("ChunkDownloaded: {:?}", msg);

    let file = get_uploading_file(&msg.uuid).await?;

    if file.receiver.cmp(my_id) != Ordering::Equal {
        eprintln!("Could not process chunk download msg, receiver in not equal to current id");
        return Ok(());
    }

    send_msg_specific(file.sender, Message::binary(msg.serialize())).await?;
    return Ok(())
}
