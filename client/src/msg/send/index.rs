use std::io::{stdin, Stdin};
use std::sync::atomic::Ordering;

use async_channel::Sender;
use colored::Colorize;
use log::trace;
use packets::communication::to::ToMsg;
use packets::initialize::pubkey::PubkeyMsg;
use packets::types::ByteMessage;
use packets::util::modes::Modes;
use packets::util::rsa::encrypt_rsa;
use tokio_tungstenite::tungstenite::Message;

use crate::encryption::rsa::get_pubkey_from_rec;
use crate::msg::send::actions::index::on_command;
use crate::util::arcs::get_curr_keypair;
use crate::util::consts::{RECEIVER, RECEIVE_INPUT, RECEIVE_TX, SEND_DISABLED};
use crate::util::msg::{print_from_msg, send_msg};
pub async fn send_msgs() -> anyhow::Result<()> {
    let keypair = get_curr_keypair().await?;

    let pem_vec = String::from_utf8(keypair.public_key_to_pem()?)?;
    let initial_msg = PubkeyMsg { pubkey: pem_vec }.serialize();

    send_msg(Message::binary(initial_msg)).await?;
    send_msg(Message::binary(Modes::WantUid.get_send(&Vec::new()))).await?;

    println!("Use /rec to change receiver\nUse /name <your name>");

    let stdin = stdin();

    let state = RECEIVE_TX.write().await;
    let tx = state.clone().unwrap();
    drop(state);

    loop {
        let res = main_loop(&stdin, &tx).await;
        if res.is_err() {
            eprintln!("Error occurred in main loop send thread: ");
            eprintln!(
                "{}",
                format!("{:?}", res.unwrap_err()).on_bright_red().black()
            );
        } 
    }
}

pub async fn main_loop(stdin: &Stdin, tx: &Sender<String>) -> anyhow::Result<()> {
    let is_disabled = SEND_DISABLED.load(Ordering::Relaxed);
    if is_disabled {
        return Ok(());
    }

    let rec = RECEIVER.read().await;
    let rec_got = rec.clone();
    let is_nothing = rec_got.is_none();

    drop(rec);
    if is_nothing {
        return Ok(());
    }

    let mut line = String::new();
    stdin.read_line(&mut line)?;

    let line = line.replace("\n", "");
    let line = line.replace("\r", "");

    let should_receive = RECEIVE_INPUT.load(Ordering::Relaxed);
    trace!("Should receive: {}", should_receive);
    if should_receive {
        let temp = tx.clone();

        trace!("Spawning tx thread...");
        let task = tokio::task::spawn(async move { temp.send(line.clone()).await });

        task.await??;
        return Ok(());
    }
    if line.starts_with("/") {
        trace!("On command");
        on_command(&line).await?;
        return Ok(());
    }

    println!("Line is '{}'", line);
    if line == "" {
        println!("Continue");
        return Ok(());
    }
    let rec_got = rec_got.clone().unwrap();

    trace!("Getting pubkey from rec...");
    let key = get_pubkey_from_rec(&rec_got).await?;
    let encrypted = encrypt_rsa(&key, &line.as_bytes().to_vec())?;

    print_from_msg("you", &line);

    let to_send = ToMsg {
        msg: encrypted,
        receiver: rec_got,
    }
    .serialize();

    trace!("Sending msg");
    send_msg(Message::Binary(to_send)).await?;
    trace!("Done.");
    return Ok(());
}
