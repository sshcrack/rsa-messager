use colored::Colorize;
use packets::{communication::error::ErrorMsg, types::WSMessage};

pub async fn on_error(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let ErrorMsg { error } = ErrorMsg::deserialize(data)?;

    eprintln!("{}", format!("Server returned error: {}", error).red());
    Ok(())
}
