use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};
use uuid::Uuid;

use crate::utils::types::Users;

pub async fn on_pubkey(msg: &Vec<u8>, my_id: &Uuid, users: &Users) -> anyhow::Result<()> {
    println!("Parsing pubkey");
    let pubkey = String::from_utf8(msg.to_vec())?;

    RsaPublicKey::from_public_key_pem(&pubkey)?;

    let mut state = users.write().await;
    let info = state.get_mut(&my_id);

    if info.is_some() {
        let i = info.unwrap();
        i.public_key = Some(pubkey);
    }

    drop(state);
    println!("Pubkey set.");
    Ok(())
}
