use log::trace;
use packets::{
    file::question::reply::FileQuestionReplyMsg,
    types::ByteMessage,
};
use warp::ws::Message;

use crate::{
    file::{consts::{PENDING_UPLOADS, UPLOADING_FILES}, tools::get_pending_file, controller::index::Controller},
    utils::tools::send_msg_specific,
};

pub async fn on_file_question_reply(data: &Vec<u8>) -> anyhow::Result<()> {
    let msg = FileQuestionReplyMsg::deserialize(&data)?;


    trace!("Getting pending file for file question reply");
    let file = get_pending_file(&msg.uuid).await?;

    let to_send = msg.serialize();
    send_msg_specific(file.receiver, Message::binary(to_send)).await?;

    if !msg.accepted {
        trace!("Deleted rejected file with id {}", msg.uuid);
        let mut state = PENDING_UPLOADS.write().await;
        state.remove(&msg.uuid);

        drop(state);
    } else {
        let uuid = msg.uuid;
        trace!("Getting pending file for file question reply else");
        let file = get_pending_file(&uuid).await?;

        trace!("Removing pending upload from {}", uuid);
        let mut state = PENDING_UPLOADS.write().await;
        state.remove(&uuid);

        drop(state);

        trace!("Adding controller to uploading files {}", uuid);
        // TODO maybe useless?
        Controller::new(&uuid, file.clone()).await?;

        let mut state = UPLOADING_FILES.write().await;
        state.insert(uuid, file);

        drop(state);
    }
    Ok(())
}
