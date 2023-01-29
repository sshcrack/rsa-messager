use std::path::Path;

use log::trace;
use packets::{file::{question::{index::FileQuestionMsg}, types::FileInfo}, types::ByteMessage, communication::error::ErrorMsg};
use warp::ws::Message;

use crate::{utils::tools::send_msg_specific, file::consts::PENDING_UPLOADS};

pub async fn on_file_question(
    data: &Vec<u8>
) -> anyhow::Result<()> {
    let msg = FileQuestionMsg::deserialize(&data)?;

    let filename = &msg.filename;
    let sender = msg.sender;

    let path = Path::new(&filename);
    let check = path.file_name();

    if check.is_none() {
        trace!("Invalid filename given ({:?})", filename);

        let err = ErrorMsg {
            error: "Invalid filename".to_string()
        }.serialize();

        send_msg_specific(sender, Message::binary(err)).await?;
        return Ok(());
    }

    let check = check.unwrap().to_str();
    if check.is_none() {
        trace!("Invalid OSString given ({:?})", filename);

        let err = ErrorMsg {
            error: "Invalid OSString".to_string()
        }.serialize();

        send_msg_specific(sender, Message::binary(err)).await?;
        return Ok(());
    }

    let state = PENDING_UPLOADS.read().await;
    let has_key = state.get(&msg.uuid).is_some();

    drop(state);
    if has_key {
        trace!("Duplicate uuid of file.");
        let err = ErrorMsg { error: "Invalid uuid of file. The same uuid is already stored.".to_string() }.serialize();

        send_msg_specific(sender, Message::binary(err)).await?;
        return Ok(());
    }

    let info = FileInfo {
        filename: msg.filename.clone(),
        receiver: msg.receiver.clone(),
        sender,
        size: msg.size,
        path: None
    };

    trace!("Storing file info {:#?}", info);

    let mut state = PENDING_UPLOADS.write().await;
    state.insert(msg.uuid, info);

    drop(state);

    let to_send = msg.serialize();
    send_msg_specific(msg.receiver, Message::binary(to_send)).await?;
    Ok(())
}
