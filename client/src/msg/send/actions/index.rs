use colored::Colorize;

use crate::util::types::TXChannel;

use super::{name::on_name, receiver::on_receiver};

pub async fn on_command(tx: &mut TXChannel, line: &str) -> anyhow::Result<()> {
    if line.eq("/rec") || line.eq("/receiver") || line.eq("/r") {
        return on_receiver(tx, line).await;
    } else if line.starts_with("/name") || line.starts_with("/n") {
        return on_name(tx, line).await;
    } else {
        println!("{}", "Unrecognized command. Either use /receiver or /name".red());
    }

    Ok(())
}
