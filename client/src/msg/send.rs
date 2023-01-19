use anyhow::anyhow;
use futures_util::{stream::SplitSink, SinkExt};
use openssl::{rsa::Rsa, pkey::Private};
use tokio_tungstenite::tungstenite::Message;

use crate::{input::receiver::select_receiver, consts::BASE_URL, encryption::rsa::encrypt};

use super::types::{WebSocketGeneral, Receiver, UserId};

pub async fn send_msgs(mut tx: SplitSink<WebSocketGeneral, Message>, my_id: UserId, receiver: Receiver, stdin: std::io::Stdin, keypair: Rsa<Private>) -> anyhow::Result<()> {
    let pem_vec = keypair.public_key_to_pem()?;
    let public_key_str = std::str::from_utf8(&pem_vec)?;

    let initial_msg = format!("setpubkey:{}", public_key_str);
    println!("Sending initial message...");
    tx.send(Message::text(initial_msg)).await?;

    loop {
        print!("Message: ");

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

            let mut state = receiver.lock().await;
            *state = new_rec.clone();

            drop(state);
            println!("Changed receiver to {}.", new_rec);
            continue;
        }

        println!("Sending message {}...", line);

        let rec = receiver.lock().await;
        let rec_got = rec.to_string();
        drop(rec);

        let pubkey_pem = get_pubkey(&rec_got).await?;
        let key = Rsa::public_key_from_pem(pubkey_pem.as_bytes())?;

        let encrypted = encrypt(key, &line)?;
        let encrypted = hex::encode(encrypted);

        tx.send(Message::Text(format!("to:{}:{}", rec_got.to_string(), encrypted))).await?;
    }
}

async fn get_pubkey(uuid: &str) -> anyhow::Result<String>{
    let list_url = format!("http://{}/pubkey?id={}", BASE_URL, uuid);

    let client = reqwest::Client::new();
    let resp = client.get(list_url.to_string())
        .send()
        .await;


    if resp.is_err() {
        eprintln!("Could not fetch from {}", list_url);
        return Err(anyhow!(resp.unwrap_err()));
    }

    let resp = resp.unwrap();
    let text = resp.text().await?;
    return Ok(text);
}