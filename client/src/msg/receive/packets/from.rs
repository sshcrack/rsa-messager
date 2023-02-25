use colored::Colorize;
use packets::{communication::{from::FromMsg, key_request::WantSymmKeyMsg}, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;

use crate::{util::{arcs::get_symm_key, msg::{print_from_msg, send_msg}}, web::user_info::get_user_info};

pub async fn on_from(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FromMsg { msg, sender } =  FromMsg::deserialize(data)?;

    let key = get_symm_key(&sender).await;
    if key.is_err() {
        println!("{}", format!("Could not get symmetric key pair for user '{}'. Sending packet again...", sender.to_string().yellow()).red());
        let packet = WantSymmKeyMsg { user: sender}.serialize();
        send_msg(Message::binary(packet)).await?;

        return Ok(());
    }

    let key = key.unwrap();

    let decrypted = key.decrypt(&msg)?;
    let msg = String::from_utf8(decrypted);

    if msg.is_err() {
        return Ok(());
    }

    let msg = msg.unwrap();

    let mut display_name = sender.to_string();
    let info = get_user_info(&sender).await?;

    if info.name.is_some() {
        let temp = info.name.unwrap();
        display_name = temp;
    }

    print_from_msg(&display_name, &msg);
    Ok(())
}
