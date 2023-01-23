use std::collections::VecDeque;

use anyhow::anyhow;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::util::types::*;
use crate::util::modes::Modes;
use crate::util::vec::decque_to_vec;

use super::packets::file_question::on_file_question;
use super::packets::file_question_reply::on_file_question_reply;
use super::packets::{from::on_from, uid::on_uid};

pub async fn receive_msgs(
    mut rx: RXChannel
) -> anyhow::Result<()> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        let res = handle(msg).await;
        if res.is_err() {
            eprintln!("----------------------------------");
            eprintln!("Error occurred while processing message: ");
            eprintln!("{:?}", res.unwrap_err());
            eprintln!("----------------------------------");
        }
    }

    Ok(())
}

pub async fn handle(
    msg: Message
) -> anyhow::Result<()> {
    let data = msg.into_data();
    let mut decque: VecDeque<u8> = VecDeque::new();
    for i in data {
        decque.push_back(i);
    }

    let mode = decque.pop_front();
    if mode.is_none() {
        println!("Received invalid message");
        return Ok(());
    }

    let mode = mode.unwrap();
    let mut data = decque_to_vec(decque);

    if Modes::From.is_indicator(&mode) {
        on_from(&mut data).await?;
        return Ok(());
    }

    if Modes::SendFileQuestion.is_indicator(&mode) {
        on_file_question(&mut data).await?;
        return Ok(());
    }

    if Modes::SendFileQuestionReply.is_indicator(&mode) {
        on_file_question_reply(&mut data).await?;
        return Ok(());
    }

    if Modes::UidReply.is_indicator(&mode) {
        on_uid(&mut data).await?;
        return Ok(());
    }

    return Err(anyhow!("Invalid packet received."));
}

