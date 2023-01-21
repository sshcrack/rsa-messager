use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{types::Users, modes::Modes, tools::{vec_to_decque, decque_to_vec}};

use super::{name::on_name, pubkey::on_pubkey, to::on_to, uid::on_uid};

pub async fn user_message(my_id: Uuid, msg: Message, users: &Users, tx: &UnboundedSender<Message>) {
    let msg = msg.into_bytes();
    let mut msg = vec_to_decque(msg);
    let mode = msg.pop_front();

    if mode.is_none() {
        eprintln!("Invalid mode.");
        return;
    }

    let mode = mode.unwrap();
    let msg = decque_to_vec(msg);

    if Modes::WantUid.is_indicator(&mode) {
        on_uid(&my_id, tx);
    }

    if Modes::To.is_indicator(&mode) {
        on_to(msg, &users).await;
        return;
    }

    if Modes::SetPubkey.is_indicator(&mode) {
        on_pubkey(&msg, &my_id, &users).await;
        return;
    }

    if Modes::Name.is_indicator(&mode) {
        on_name(&msg, &my_id, &users).await;
        return;
    }
}