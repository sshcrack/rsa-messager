use std::collections::VecDeque;
use std::io::stdin;
use std::sync::atomic::Ordering;

use openssl::rsa::Rsa;
use tokio_tungstenite::tungstenite::Message;

use crate::msg::send::actions::index::on_command;
use crate::util::arcs::get_curr_keypair;
use crate::util::consts::{SEND_DISABLED, RECEIVER, RECEIVE_INPUT, RECEIVE_TX};
use crate::util::modes::Modes;
use crate::util::msg::{send_msg, print_from_msg};
use crate::util::vec::{vec_to_decque, decque_to_vec};
use crate::{
    encryption::rsa::encrypt,
    web::user_info::get_user_info,
};
pub async fn send_msgs() -> anyhow::Result<()> {
    let keypair = get_curr_keypair().await?;
    let pem_vec = keypair.public_key_to_pem()?;

    let initial_msg = Modes::SetPubkey.get_send(&pem_vec);

    let mut get_uid = Vec::new();
    get_uid.push(Modes::WantUid.get_indicator());

    send_msg(Message::binary(initial_msg)).await?;
    send_msg(Message::binary(get_uid)).await?;

    println!("Use /rec to change receiver\nUse /name <your name>");

    let stdin = stdin();
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
        println!("Disabled is {} line is {}", is_disabled, line);
        if should_receive {
            println!("Sending line {} to tx", line.clone());
            let state = RECEIVE_TX.write().await;
            let e = state.as_ref().unwrap().send(line.clone());

            drop(state);
            println!("Dropped");
            println!("{:?}", e);
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

        let mut merged = VecDeque::new();
        let mut rec_b = vec_to_decque(rec_got.as_bytes().to_vec());
        let mut encrypted_b = vec_to_decque(encrypted.to_vec());

        merged.append(&mut rec_b);
        merged.append(&mut encrypted_b);

        let merged = decque_to_vec(merged);
        let to_send = Modes::To.get_send(&merged);

        send_msg(Message::Binary(to_send)).await?;

    }
}
