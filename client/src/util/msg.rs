use std::sync::atomic::Ordering;

use anyhow::anyhow;
use colored::Colorize;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use super::consts::{TX_CHANNEL, ABORT_TX, SEND_DISABLED};


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

pub async fn lock_input() -> anyhow::Result<()> {
    SEND_DISABLED.store(true, Ordering::Relaxed);

    let rx = ABORT_TX.read().await;
    if rx.is_none() {
        drop(rx);

        SEND_DISABLED.store(false, Ordering::Relaxed);
        return Err(anyhow!("Could not lock input, rx is null."));
    }

    let rx_u = rx.as_ref().unwrap();
    let res = rx_u.send(true);
    drop(rx);

    if res.is_err() {
        SEND_DISABLED.store(false, Ordering::Relaxed);
    }
    res?;


    Ok(())
}


pub async fn release_input() -> anyhow::Result<()> {
    SEND_DISABLED.store(false, Ordering::Relaxed);

    let rx = ABORT_TX.read().await;
    if rx.is_none() {
        drop(rx);
        return Err(anyhow!("Could not release input, rx is null."));
    }
    let rx_u = rx.as_ref().unwrap();
    rx_u.send(false)?;

    drop(rx);

    Ok(())
}