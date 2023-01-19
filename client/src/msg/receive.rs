use futures_util::{StreamExt, stream::{SplitStream}};
use openssl::{rsa::Rsa, pkey::Private};

use crate::encryption::rsa::decrypt;

use super::types::{WebSocketGeneral, UserId};

pub async fn receive_msgs(mut rx: SplitStream<WebSocketGeneral>, curr_id: UserId, keypair: Rsa<Private>) -> anyhow::Result<()> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if !msg.is_text() {
            continue;
        }

        let msg_txt = msg.to_text().unwrap().to_string();
        if msg_txt.starts_with("UID:") {
            let mut state = curr_id.write().unwrap();
            let rec = msg_txt.replace("UID:", "").to_string();

            *state = Some(rec.clone());

            drop(state);

            println!("Current id set to {}", rec);
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

            let encrypted_hex = parts.join("$");
            let encrypted = hex::decode(encrypted_hex)?;

            let decrypted = decrypt(keypair.clone(), encrypted)?;
            let msg = std::str::from_utf8(&decrypted)?;

            println!("[{}]: {}", from_id, msg);
        }
    }

    Ok(())
}

