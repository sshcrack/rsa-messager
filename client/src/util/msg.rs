use std::sync::atomic::Ordering;

use colored::Colorize;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::util::consts::{RECEIVE_INPUT, RECEIVE_RX};

use super::consts::TX_CHANNEL;


pub fn print_from_msg(display_name: &str, msg: &str) {
    println!(
        "{}{}{} {}",
        "[".to_string().bright_black(),
        display_name,
        "]:".to_string().bright_black(),
        msg.green().bold()
    );
}

pub async fn send_msg(msg: Message) -> anyhow::Result<()> {
    let mut tx_o = TX_CHANNEL.lock().await;
    let tx = tx_o.as_mut().unwrap();

    let e = tx.send(msg).await;

    drop(tx);
    e?;

    Ok(())
}

pub async fn get_input() -> anyhow::Result<String> {
    RECEIVE_INPUT.store(true, Ordering::Relaxed);

    let state = RECEIVE_RX.write().await;
    let rx = state.clone().unwrap();

    drop(state);

    let e = rx.recv().await;
    RECEIVE_INPUT.store(false, Ordering::Relaxed);

    Ok(e?)
}