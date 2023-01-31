use anyhow::anyhow;
use packets::util::modes::Modes;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::vec::{vec_to_decque, decque_to_vec};

use super::{name::on_name, pubkey::on_pubkey, to::on_to, uid::on_uid, question::{reply::on_file_question_reply, question::on_file_question}, file::{downloaded::on_chunk_downloaded, abort::on_chunk_abort}};

pub async fn user_message(my_id: Uuid, msg: Message, tx: &UnboundedSender<Message>) -> anyhow::Result<()> {
    let msg = msg.into_bytes();
    let mut msg = vec_to_decque(msg);
    let mode = msg.pop_front();

    if mode.is_none() {
        eprintln!("Invalid mode.  (is none)");
        return Err(anyhow!("Invalid mode."));
    }

    let mode = mode.unwrap();
    let msg = decque_to_vec(msg);

    if Modes::WantUid.is_indicator(&mode) {
        return on_uid(&my_id, tx);
    }

    if Modes::To.is_indicator(&mode) {
        return on_to(msg, &my_id).await;
    }

    if Modes::SetPubkey.is_indicator(&mode) {
        return on_pubkey(&msg, &my_id).await;
    }

    if Modes::Name.is_indicator(&mode) {
        return on_name(&msg, &my_id).await;
    }

    if Modes::SendFileQuestion.is_indicator(&mode) {
        return on_file_question(&msg).await;
    }

    if Modes::SendFileQuestionReply.is_indicator(&mode) {
        return on_file_question_reply(&msg).await;
    }
    if Modes::SendFileChunkDownloaded.is_indicator(&mode) {
        return on_chunk_downloaded(&msg, &my_id).await;
    }

    if Modes::SendFileAbort.is_indicator(&mode) {
        return on_chunk_abort(&msg, &my_id).await;
    }

    Err(anyhow!("Invalid packet mode."))
}