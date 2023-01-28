use std::cmp::Ordering;

use log::trace;
use packets::{file::processing::ready::ChunkReadyMsg, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::{file::tools::get_uploading_file, utils::tools::send_msg_specific};

pub async fn on_chunk_ready(data: &Vec<u8>, my_id: &Uuid) -> anyhow::Result<()> {
    let msg = ChunkReadyMsg::deserialize(data)?;
    trace!("ChunkReady: {:?}", msg);

    let file = get_uploading_file(&msg.uuid).await?;

    if file.sender.cmp(my_id) != Ordering::Equal {
        eprintln!("Could not process chunk ready msg, sender in not equal to current id");
        return Ok(());
    }

    send_msg_specific(file.receiver, Message::binary(msg.serialize())).await?;
    return Ok(())
}
