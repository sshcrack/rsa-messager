use crate::{encryption::rsa::decrypt, util::{tools::uuid_from_vec, arcs::get_curr_keypair, msg::print_from_msg}, web::user_info::get_user_info};

pub async fn on_from(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let from_id = uuid_from_vec(data)?;
    let keypair = get_curr_keypair().await?;

    let decrypted = decrypt(keypair.clone(), data.to_vec())?;
    let msg = String::from_utf8(decrypted);

    if msg.is_err() {
        println!("Msg byte");
        return Ok(());
    }

    let msg = msg.unwrap();

    let mut display_name = from_id.to_string();
    let info = get_user_info(&from_id).await?;

    if info.name.is_some() {
        let temp = info.name.unwrap();
        display_name = temp;
    }

    print_from_msg(&display_name, &msg);
    Ok(())
}
