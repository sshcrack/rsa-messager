use std::io::stdin;
use std::sync::atomic::Ordering;

use packets::communication::to::ToMsg;
use packets::initialize::pubkey::PubkeyMsg;
use packets::types::ByteMessage;
use packets::util::modes::Modes;
use packets::util::rsa::encrypt_rsa;
use tokio_tungstenite::tungstenite::Message;

use crate::encryption::rsa::get_pubkey_from_rec;
use crate::msg::send::actions::index::on_command;
use crate::util::arcs::get_curr_keypair;
use crate::util::consts::{SEND_DISABLED, RECEIVER, RECEIVE_INPUT, RECEIVE_TX};
use crate::util::msg::{send_msg, print_from_msg};
pub async fn send_msgs() -> anyhow::Result<()> {
    let keypair = get_curr_keypair().await?;

    let pem_vec = String::from_utf8(keypair.public_key_to_pem()?)?;
    let initial_msg = PubkeyMsg{ pubkey: pem_vec }.serialize();

    send_msg(Message::binary(initial_msg)).await?;
    send_msg(Message::binary(Modes::WantUid.get_send(&Vec::new()))).await?;

    println!("Use /rec to change receiver\nUse /name <your name>");

    let stdin = stdin();

    let state = RECEIVE_TX.write().await;
    let tx = state.clone().unwrap();
    drop(state);

    loop {
        let is_disabled = SEND_DISABLED.load(Ordering::Relaxed);
        if is_disabled {
            continue;
        }

        let rec = RECEIVER.read().await;
        let rec_got = rec.clone();
        let is_nothing = rec_got.is_none();

        drop(rec);
        if is_nothing {
            continue;
        }


        let mut line = String::new();

        stdin.read_line(&mut line)?;
        let line = line.replace("\n", "");
        let line = line.replace("\r", "");

        let should_receive = RECEIVE_INPUT.load(Ordering::Relaxed);
        if should_receive {
            let temp = tx.clone();

            tokio::task::spawn(async move {
                temp.send(line.clone()).await
            });

            tokio::task::yield_now().await;
            continue;
        }

        if line.starts_with("/") {
            on_command(&line).await?;
            continue;
        }

        if line == "" {
            continue;
        }

        let rec_got = rec_got.clone().unwrap();


        let key = get_pubkey_from_rec(&rec_got).await?;
        let encrypted = encrypt_rsa(&key, &line.as_bytes().to_vec())?;

        print_from_msg("you", &line);

        let to_send = ToMsg {
            msg: encrypted,
            receiver: rec_got
        }.serialize();

        send_msg(Message::Binary(to_send)).await?;

    }
}
