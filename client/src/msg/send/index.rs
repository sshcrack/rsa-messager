use std::io::stdin;
use std::sync::atomic::Ordering;

use openssl::rsa::Rsa;
use packets::communication::to::ToMsg;
use packets::initialize::pubkey::PubkeyMsg;
use packets::types::WSMessage;
use packets::util::modes::Modes;
use tokio_tungstenite::tungstenite::Message;

use crate::msg::send::actions::index::on_command;
use crate::util::arcs::get_curr_keypair;
use crate::util::consts::{SEND_DISABLED, RECEIVER, RECEIVE_INPUT, RECEIVE_TX};
use crate::util::msg::{send_msg, print_from_msg};
use crate::{
    encryption::rsa::encrypt,
    web::user_info::get_user_info,
};
pub async fn send_msgs() -> anyhow::Result<()> {
    let keypair = get_curr_keypair().await?;

    let pem_vec = keypair.public_key_to_pem()?;
    let pem_vec = String::from_utf8(pem_vec)?;

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

        print_from_msg("you", &line);

        let to_send = ToMsg {
            msg: encrypted,
            receiver: rec_got
        }.serialize();

        send_msg(Message::Binary(to_send)).await?;

    }
}
