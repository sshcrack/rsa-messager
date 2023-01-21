use std::error::Error;
use std::fmt;
use std::io::stdin;
use std::process::exit;

use anyhow::anyhow;
use clap::Parser;
use futures_util::StreamExt;
use tokio::task;
use tokio_tungstenite::connect_async;

use crate::encryption::rsa::generate;
use crate::msg::receive::index::receive_msgs;
use crate::msg::send::index::send_msgs;
use crate::util::consts::{BASE_URL, KEYPAIR};
use crate::util::types::{Args};

mod encryption;
mod input;
mod msg;
mod util;
mod web;

#[derive(Debug)]
struct ReqwestError {
    orig: reqwest::Error,
}

impl fmt::Display for ReqwestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Error for ReqwestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.orig)
    }
}

#[tokio::main]
async fn main() {
    let res = task::spawn(async move {
        let e = _async_main().await;
        if e.is_err() {
            eprintln!("Main Run Error:");
            eprintln!("{:#?}", e.unwrap_err());
            exit(-1)
        }
    })
    .await;
    if res.is_err() {
        eprintln!("Main Run Error:");
        eprintln!("{:#?}", res.unwrap_err());
        exit(-1)
    }
}

async fn _async_main() -> anyhow::Result<()> {
    let args = Args::parse();
    let base_url = args.address.unwrap_or("localhost:3030".to_string());

    let mut state = BASE_URL.write().await;
    *state = base_url.clone();

    drop(state);

    println!("Generating keypair...");
    let keypair = generate();

    let mut state = KEYPAIR.write().await;
    *state = Some(keypair.clone());

    drop(state);

    let ws_url = format!("ws://{}/chat", base_url);

    println!("Connecting to {}...", ws_url.to_string());

    let (ws_stream, _) = connect_async(ws_url.to_string()).await?;

    let (tx, rx) = ws_stream.split();
    let stdin = stdin();


    let receive = tokio::spawn(async move {
        let res = receive_msgs(rx).await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("RecErr: {}", err);
            return Err(err);
        }

        return Ok(());
    });

    let send_f = tokio::spawn(async move {
        let res = send_msgs(tx, stdin).await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("SendErr: {}", err);
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
            eprintln!("Rec: {}", err);

            return Err(anyhow!("Join Error idk"));
        }

        let res = res.unwrap();
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Rec: {}", err);

            return Err(err);
        }
    }

    if send_f.is_finished() {
        let res = send_f.await;

        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Rec: {}", err);

            return Err(anyhow!("Joinm Error idk"));
        }

        let res = res.unwrap();
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("Send: {}", err);

            return Err(err);
        }
    }
    Ok(())
}
