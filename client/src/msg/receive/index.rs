use std::collections::VecDeque;

use anyhow::anyhow;
use futures_util::StreamExt;
use packets::util::modes::Modes;
use packets::util::vec::decque_to_vec;
use tokio_tungstenite::tungstenite::Message;

use crate::util::types::*;

use super::packets::error::on_error;
use super::packets::file::file_question::on_file_question;
use super::packets::file::file_question_reply::on_file_question_reply;
use super::packets::file::start_processing::on_start_processing;
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

    if Modes::Error.is_indicator(&mode) {
        on_error(&mut data).await?;
        return Ok(());
    }

    if Modes::SendFileStartProcessing.is_indicator(&mode) {
        on_start_processing(&mut data).await?;
        return Ok(());
    }

    return Err(anyhow!("Invalid packet received."));
}

