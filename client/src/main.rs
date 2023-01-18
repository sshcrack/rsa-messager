use std::{process::exit, sync::{Arc, RwLock}, os::windows::process, io::stdin};

use futures_util::{SinkExt, StreamExt, stream::{SplitStream, SplitSink}};
use inquire::{Select, Text};
use tokio::{task, net::TcpStream, io::stdin};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream, tungstenite::Message};

type UserId = Arc<RwLock<Option<String>>>;
type WebSocketGeneral = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::main]
async fn main() {
    let res = task::spawn(_async_main()).await;
    if res.is_err() {
        eprintln!("{:#?}", res.unwrap_err());
        exit(-1)
    }
}

async fn _async_main() -> anyhow::Result<()> {
    let base_url = "localhost:3030";
    let list_url = format!("http://{}/list", base_url);


    let (ws_stream, _) = connect_async(format!("ws://{}/chat", base_url)).await?;
    let curr_id = UserId::default();
    let curr_id_second = curr_id.clone();


    let client = reqwest::Client::new();
    let resp = client.get(list_url.to_string())
        .send()
        .await;


    if resp.is_err() {
        eprintln!("Could not fetch from {}", list_url);
        eprintln!("");
        return Ok(());
    }

    let resp = resp.unwrap();
    let text = resp.text().await?;

    let available: Vec<String> = serde_json::from_str(&text)?;
    println!("Available clients are: {:#?}", available);

    let receiver = Select::new("Receiver:", available)
        .prompt()?;

    let (tx, rx) = ws_stream.split();
    let receive_thread = tokio::spawn(_receive_msgs(rx, curr_id, receiver.clone()));
    let send_thread = tokio::spawn(_send_msgs(tx, curr_id_second, receiver.clone()));

    // Block rust from exiting
    while !receive_thread.is_finished() || !send_thread.is_finished() { }
    Ok(())
}


async fn _receive_msgs(mut rx: SplitStream<WebSocketGeneral>, curr_id: UserId, receiver: String) -> anyhow::Result<()> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if !msg.is_text() {
            continue;
        }

        println!("Incoming: {}",msg);
        let msg_txt = msg.to_text().unwrap().to_string();
        if msg_txt.starts_with("UID:") {
            let mut state = curr_id.write().unwrap();
            *state = Some(msg_txt.replace("UID:", "").to_string());

            println!("Current id set to {:#?}", curr_id.read().unwrap());
            continue;
        }

        if msg_txt.starts_with("from:") {
            let msg_txt = msg_txt.replace("from:", "");
            let mut parts: Vec<&str> = msg_txt.rsplit("$").collect();
            let parts_clone = parts.clone();

            let from_id = parts_clone.get(0);
            if from_id.is_none() {
                eprintln!("Invalid format of message {}", msg_txt);
                continue;
            }

            let from_id = from_id.unwrap();
            parts.remove(0);

            let msg = parts.join("$");
            println!("Received [{}]: {}", from_id, msg);
        }
    }

    Ok(())
}



async fn _send_msgs(mut tx: SplitSink<WebSocketGeneral, Message>, _curr_id: UserId, _receiver: String) -> anyhow::Result<()> {
    loop {
        println!("Asking you for message:");
        let line = std::io::stdin().lines().next().unwrap().unwrap();
        println!("Line {}:", line)
        /*let msg = Text::new("Message:").prompt()?;
        println!("Sending message {}", msg);

        tx.send(Message::Text(msg)).await?;*/
    }
}