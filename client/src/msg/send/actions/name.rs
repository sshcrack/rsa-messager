use colored::Colorize;
use tokio_tungstenite::tungstenite::Message;

use crate::util::{modes::Modes, msg::send_msg};

pub async fn on_name(line: &str) -> anyhow::Result<()> {
    let new_name = line.split(" ");
    let new_name = Vec::from_iter(new_name.skip(1)).join(" ");

    if new_name.len() <= 3 || new_name.len() > 20 {
        println!("{}", "Name length has to be between 4 and 20 characters.".red());
        return Ok(())
    }

    let name_byte = new_name.clone().into_bytes();
    let to_send = Modes::Name.get_send(&name_byte);


    send_msg(Message::Binary(to_send)).await?;

    let e = format!("Name changed to: {}", new_name).bright_blue();
    println!("{}", e);
    return Ok(());
}
