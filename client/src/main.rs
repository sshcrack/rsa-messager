use std::{process::exit, sync::{Arc, RwLock}};

use futures_util::{SinkExt, StreamExt, stream::{SplitStream, SplitSink}};
use inquire::{Select, Text};
use tokio::{task, net::TcpStream};
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
    let stdin = std::io::stdin();

    let receive = tokio::spawn(async move {
        let res = _receive_msgs(rx, curr_id).await;
        if res.is_err() {
            eprintln!("{}", res.unwrap_err());
            return Ok(());
        }

        return Ok(());
    });
    let send_f = tokio::spawn(async move {
        _send_msgs(tx, receiver.clone(), stdin).await
    });


    println!("Send await");
    task::yield_now().await;
    let rec_res = receive.await;

    if rec_res.is_err() {
        eprintln!("Join err");
        return Ok(());
    }
    let rec_res = rec_res.unwrap();

    if rec_res.is_err() {
        return Err(rec_res.unwrap_err());
    }

    let send_res = send_f.await;


    if send_res.is_err() {
        eprintln!("Join err");
        return Ok(());
    }

    let send_res = send_res.unwrap();

    if send_res.is_err() {
        return Err(send_res.unwrap_err());
    }



    println!("Done");
    Ok(())
}


async fn _receive_msgs(mut rx: SplitStream<WebSocketGeneral>, curr_id: UserId) -> anyhow::Result<()> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if !msg.is_text() {
            continue;
        }

        let msg_txt = msg.to_text().unwrap().to_string();
        if msg_txt.starts_with("UID:") {
            let mut state = curr_id.write().unwrap();
            *state = Some(msg_txt.replace("UID:", "").to_string());

            drop(state);

            println!("Current id set to {:#?}", curr_id.read().unwrap());
            continue;
        }

        if msg_txt.starts_with("from:") {
            let msg_txt = msg_txt.replace("from:", "");
            let mut parts: Vec<&str> = msg_txt.split(":").collect();
            let parts_clone = parts.clone();

            let from_id = parts_clone.get(0);
            if from_id.is_none() {
                eprintln!("Invalid format of message {}", msg_txt);
                continue;
            }

            let from_id = from_id.unwrap();
            parts.remove(0);

            let msg = parts.join("$");
            println!("[{}]: {}", from_id, msg);
        }
    }

    Ok(())
}



async fn _send_msgs(mut tx: SplitSink<WebSocketGeneral, Message>, receiver: String, stdin: std::io::Stdin) -> anyhow::Result<()> {
    println!("Msgs");
    loop {
        println!("Asking you for message:");
        print!("Message: ");

        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let line = line.replace("\n", "");
        let line = line.replace("\r", "");

        println!("Sending message {}", line);

        tx.send(Message::Text(format!("to:{}:{}", receiver, line))).await?;
    }
}