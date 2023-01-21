use std::sync::atomic::Ordering;

use colored::Colorize;
use futures_util::{stream::SplitSink, SinkExt};
use openssl::{rsa::Rsa, pkey::Private};
use tokio_tungstenite::tungstenite::Message;

use crate::{input::receiver::select_receiver, encryption::rsa::encrypt, web::user_info::get_user_info};

use super::types::{WebSocketGeneral, Receiver, UserId, SendDisabled};

pub async fn send_msgs(mut tx: SplitSink<WebSocketGeneral, Message>, my_id: UserId, receiver: Receiver, stdin: std::io::Stdin, keypair: Rsa<Private>, send_disabled: SendDisabled) -> anyhow::Result<()> {
    let pem_vec = keypair.public_key_to_pem()?;
    let public_key_str = std::str::from_utf8(&pem_vec)?;

    let initial_msg = format!("setpubkey:{}", public_key_str);
    tx.send(Message::text(initial_msg)).await?;
    tx.send(Message::text("getuid")).await?;

    println!("Use /rec to change receiver\nUse /name <your name>");
    loop {
        let is_disabled = send_disabled.load(Ordering::Relaxed);
        if is_disabled { continue; }

        let rec = receiver.read().await;
        let rec_got = rec.clone();
        let is_nothing = rec_got.is_none();

        drop(rec);
        if is_nothing { continue; }

        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let line = line.replace("\n", "");
        let line = line.replace("\r", "");

        if line.eq("/rec") || line.eq("/receiver") {
            let new_rec = select_receiver(my_id.clone()).await;
            if new_rec.is_err() {
                println!("Rec Error: {:#?}", new_rec.unwrap_err());
                continue;
            }

            let new_rec = new_rec.unwrap();

            let mut state = receiver.write().await;
            *state = Some(new_rec.clone());

            drop(state);
            continue;
        }

        if line.starts_with("/name ") {
            let new_name = line.replace("/name ", "");
            tx.send(Message::Text(format!("name:{}", new_name))).await?;

            println!("Name changed to: {}", new_name);
            continue;
        }

        let rec_got = rec_got.clone().unwrap();


        let info = get_user_info(&rec_got).await?;
        let pubkey_pem = info.public_key;

        if pubkey_pem.is_none() {
            println!("Could not get pubkey of receiver.");
            continue;

        }

        let pubkey_pem = pubkey_pem.unwrap();
        let key = Rsa::public_key_from_pem(pubkey_pem.as_bytes())?;

        let encrypted = encrypt(key, &line)?;
        let encrypted = hex::encode(encrypted);

        println!("{}you{} {}", "[".to_string().bright_black(), "]:".to_string().bright_black(), line.green().bold());

        tx.send(Message::Text(format!("to:{}:{}", rec_got.to_string(), encrypted))).await?;
    }
}