use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};
use uuid::Uuid;

use crate::utils::types::Users;

pub async fn on_pubkey(msg: &Vec<u8>, my_id: &Uuid, users: &Users) {
    println!("Parsing pubkey");
    let pubkey = String::from_utf8(msg.to_vec());
    if pubkey.is_err() {
        eprintln!("Could not parse public key.");
        return;
    }

    let pubkey = pubkey.unwrap();
    let pubkey_test = RsaPublicKey::from_public_key_pem(&pubkey);
    if pubkey_test.is_err() {
        eprint!("Invalid public key given.");
        return;
    }

    let mut state = users.write().await;
    let info = state.get_mut(&my_id);

    if info.is_some() {
        let i = info.unwrap();
        i.public_key = Some(pubkey);
    }

    drop(state);
    println!("Pubkey set.");
}
