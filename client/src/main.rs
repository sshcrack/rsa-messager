use std::process::exit;

use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use futures_util::StreamExt;
use packets::initialize::name::NameMsg;
use packets::types::ByteMessage;
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use util::consts::{RECEIVE_RX, RECEIVE_TX};

use crate::encryption::rsa::generate;
use crate::msg::receive::index::receive_msgs;
use crate::msg::send::index::send_msgs;
use crate::util::consts::{BASE_URL, CONCURRENT_THREADS, KEYPAIR, TX_CHANNEL, USE_TLS};
use crate::util::msg::send_msg;
use crate::util::types::Args;
use crate::web::prefix::get_ws_protocol;

mod encryption;
mod file;
mod input;
mod msg;
mod util;
mod web;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let res = task::spawn(async move {
        let e = _async_main().await;
        if e.is_err() {
            eprintln!("{}", format!("Main Run Error:").on_red());
            eprintln!("{:#?}", e.unwrap_err());
            exit(-1)
        }
    })
    .await;
    if res.is_err() {
        eprintln!("{}", format!("Main Run Error:").on_red());
        eprintln!("{:#?}", res.unwrap_err());
        exit(-1)
    }
}

async fn _async_main() -> anyhow::Result<()> {
    initialize_consts().await;

    let args = Args::parse();
    let mut base_url = args.address;
    let mut secure = false;

    if base_url.starts_with("wss://") || base_url.starts_with("https://") {
        base_url = base_url.replace("wss://", "");
        base_url = base_url.replace("https://", "");

        secure = true;
    }

    base_url = base_url.replace("http://", "");
    base_url = base_url.replace("ws://", "");

    if secure {
        println!("{}", format!("Using secure connection...").blue());
    }

    let mut state = USE_TLS.write().await;
    *state = secure;

    drop(state);

    let mut state = BASE_URL.write().await;
    *state = base_url.clone();

    drop(state);

    let mut state = CONCURRENT_THREADS.write().await;
    *state = args.threads.unwrap_or(64 as usize) as u64;

    drop(state);

    println!("{}", format!("Generating RSA keypair...").green());
    let keypair = generate();

    let mut state = KEYPAIR.write().await;
    *state = Some(keypair.clone());

    drop(state);

    let ws_protocol = get_ws_protocol().await;
    let ws_url = format!("{}//{}/chat", ws_protocol, base_url);

    println!(
        "{}",
        format!("Connecting to {} ...", ws_url.to_string()).yellow()
    );

    let (ws_stream, _) = connect_async(ws_url.to_string()).await?;

    let (tx, rx) = ws_stream.split();
    let mut state = TX_CHANNEL.lock().await;
    *state = Some(tx);

    drop(state);

    if args.name.is_some() {
        let initial_name = args.name.unwrap();
        println!("{}", format!("Setting initial name...").bright_yellow());
        send_msg(Message::binary(NameMsg { name: initial_name }.serialize())).await?;
    }

    let receive = tokio::spawn(async move {
        let res = receive_msgs(rx).await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("RecErr: {:?}", err);
            return Err(err);
        }

        return Ok(());
    });

    let send_f = tokio::spawn(async move {
        let res = send_msgs().await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("SendErr: {:?}", err);
            return Err(err);
        }

        return Ok(());
    });

    task::yield_now().await;
    while !receive.is_finished() && !send_f.is_finished() {}

    if receive.is_finished() {
        let res = receive.await;

        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Rec: {:?}", err);

            return Err(anyhow!("Join Error idk"));
        }

        let res = res.unwrap();
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Rec: {:?}", err);

            return Err(err);
        }
    }

    if send_f.is_finished() {
        let res = send_f.await;

        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Rec: {:?}", err);

            return Err(anyhow!("Joinm Error idk"));
        }

        let res = res.unwrap();
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Send: {:?}", err);

            return Err(err);
        }
    }
    Ok(())
}

pub async fn initialize_consts() {
    let (tx, rx) = async_channel::unbounded::<String>();

    let mut state = RECEIVE_TX.write().await;
    *state = Some(tx);

    drop(state);

    let mut state = RECEIVE_RX.write().await;
    *state = Some(rx);

    drop(state);
}
