
use std::{process::exit};
use std::error::Error;
use std::fmt;

use anyhow::anyhow;
use futures_util::{StreamExt, lock::Mutex};
use tokio::task;
use tokio_tungstenite::connect_async;

use crate::encryption::rsa::generate;
use crate::msg::receive::receive_msgs;
use crate::msg::send::send_msgs;
use crate::msg::types::{UserId, Receiver};
use crate::{input::receiver::select_receiver, consts::BASE_URL};

mod encryption;
mod input;
mod consts;
mod msg;



#[derive(Debug)]
struct ReqwestError {
    orig: reqwest::Error
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
    println!("Spawning main thread...");
    let res = task::spawn(_async_main()).await;
    if res.is_err() {
        eprintln!("Main Run Error:");
        eprintln!("{:#?}", res.unwrap_err());
        exit(-1)
    }
}

async fn _async_main() -> anyhow::Result<()> {
    println!("Generating keypair...");
    let keypair_org = generate();

    let ws_url = format!("ws://{}/chat", BASE_URL);

    println!("Connecting to {}...", ws_url.to_string());
    let (ws_stream, _) = connect_async(ws_url.to_string()).await?;
    let curr_id = UserId::default();


    let receiver_str = select_receiver(curr_id.clone()).await?;
    let receiver = Receiver::new(Mutex::new(receiver_str));

    let (tx, rx) = ws_stream.split();
    let stdin = std::io::stdin();

    let keypair = keypair_org.clone();
    let temp = curr_id.clone();
    let receive = tokio::spawn(async move {
        let res = receive_msgs(rx, temp, keypair).await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("RecErr: {}", err);
            return Err(err);
        }

        return Ok(());
    });

    let keypair = keypair_org.clone();
    let temp = curr_id.clone();
    let send_f = tokio::spawn(async move {
        let res = send_msgs(tx, temp, receiver, stdin, keypair).await;
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

            return Err(anyhow!("Joinm Error idk"));
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
