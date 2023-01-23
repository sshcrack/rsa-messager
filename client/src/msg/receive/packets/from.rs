use packets::{communication::from::FromMsg, types::WSMessage};

use crate::{encryption::rsa::decrypt, util::{arcs::get_curr_keypair, msg::print_from_msg}, web::user_info::get_user_info};

pub async fn on_from(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FromMsg { msg, sender } =  FromMsg::deserialize(data)?;
    let keypair = get_curr_keypair().await?;

    let decrypted = decrypt(keypair.clone(), msg)?;
    let msg = String::from_utf8(decrypted);

    if msg.is_err() {
        println!("Msg byte");
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
