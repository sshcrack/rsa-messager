use colored::Colorize;
use packets::{initialize::name::NameMsg, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;

use crate::util::msg::send_msg;

pub async fn on_name(line: &str) -> anyhow::Result<()> {
    let new_name = line.split(" ");
    let new_name = Vec::from_iter(new_name.skip(1)).join(" ");

    if new_name.len() <= 3 || new_name.len() > 20 {
        println!("{}", "Name length has to be between 4 and 20 characters.".red());
        return Ok(())
    }

    let to_send = NameMsg {
        name: new_name.clone()
    }.serialize();

    send_msg(Message::Binary(to_send)).await?;

    let e = format!("Name changed to: {}", new_name).bright_blue();
    println!("{}", e);
    return Ok(());
}
