use std::collections::VecDeque;

use anyhow::anyhow;
use futures_util::{stream::SplitStream, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::util::types::*;
use crate::util::{modes::Modes, tools::decque_to_vec};

use super::packets::{from::on_from, uid::on_uid};

pub async fn receive_msgs(
    mut rx: SplitStream<WebSocketGeneral>
) -> anyhow::Result<()> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        let res = handle(msg).await;
        if res.is_err() {
            eprintln!("----------------------------------");
            eprintln!("Error occurred while processing message: ");
            eprintln!("{}", res.unwrap_err());
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

    let text_parse = String::from_utf8(data.clone());
    if text_parse.is_ok() {
        let msg = text_parse.unwrap();
        if Modes::UidReply.is_indicator(&mode) {
            on_uid(&msg).await?;
            return Ok(());
        }
    }

    if Modes::From.is_indicator(&mode) {
        on_from(&mut data).await?;
        return Ok(());
    }

    return Err(anyhow!("Invalid packet received."));
}
