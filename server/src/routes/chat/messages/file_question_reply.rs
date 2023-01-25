use log::trace;
use packets::{
    file::question::reply::FileQuestionReplyMsg,
    types::ByteMessage,
};
use warp::ws::Message;

use crate::{
    file::{consts::{PENDING_UPLOADS, UPLOADING_FILES}, tools::get_pending_file, controller::index::Controller},
    utils::{tools::send_msg_specific, types::Users},
};

pub async fn on_file_question_reply(data: &Vec<u8>, users: &Users) -> anyhow::Result<()> {
    let msg = FileQuestionReplyMsg::deserialize(&data)?;


    let file = get_pending_file(msg.uuid).await?;

    let to_send = msg.serialize();
    send_msg_specific(file.receiver, users, Message::binary(to_send)).await?;

    if !msg.accepted {
        trace!("Deleted rejected file with id {}", msg.uuid);
        let mut state = PENDING_UPLOADS.write().await;
        state.remove(&msg.uuid);

        drop(state);
    } else {
        let uuid = msg.uuid;
        let file = get_pending_file(uuid).await?;

        trace!("Removing pending upload from {}", uuid);
        let mut state = PENDING_UPLOADS.write().await;
        state.remove(&uuid);

        drop(state);

        trace!("Adding controller to uploading files {}", uuid);
        let controller = Controller::new(&uuid, &users, file).await?;

        let mut state = UPLOADING_FILES.write().await;
        state.insert(uuid, controller);

        drop(state);
    }
    Ok(())
}
