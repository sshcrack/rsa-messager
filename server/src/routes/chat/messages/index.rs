use anyhow::anyhow;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{types::Users, modes::Modes, vec::{vec_to_decque, decque_to_vec}};

use super::{name::on_name, pubkey::on_pubkey, to::on_to, uid::on_uid, file_question::on_file_question, file_question_reply::on_file_question_reply};

pub async fn user_message(my_id: Uuid, msg: Message, users: &Users, tx: &UnboundedSender<Message>) -> anyhow::Result<()> {
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
        return on_to(msg, &my_id, &users).await;
    }

    if Modes::SetPubkey.is_indicator(&mode) {
        return on_pubkey(&msg, &my_id, &users).await;
    }

    if Modes::Name.is_indicator(&mode) {
        return on_name(&msg, &my_id, &users).await;
    }

    if Modes::SendFileQuestion.is_indicator(&mode) {
        return on_file_question(&msg, &my_id, &users).await;
    }

    if Modes::SendFileQuestionReply.is_indicator(&mode) {
        return on_file_question_reply(&msg,  &users).await;
    }

    Err(anyhow!("Invalid packet mode."))
}