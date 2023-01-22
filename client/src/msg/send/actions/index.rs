use colored::Colorize;

use super::{name::on_name, receiver::on_receiver, send::on_send};

pub fn is_command(line: &str, aliases: Vec<&str>) -> bool{
    return aliases.iter().any(|e|{
        let alias = format!("/{e}");
        let with_space = format!("{} ", alias);

        return line.eq(&alias) || line.starts_with(&with_space);
    })
}
pub fn get_help_str() -> String {
    let rec_cmd = format!("{}: {}", "/receiver".bold().bright_blue(), "Change the user you want to write a message to / send a file to.".bright_black());
    let name_cmd = format!("{} {}: {}", "/name".bold().bright_blue(), "<name>".bright_blue(), "Changes your display name to the given name".bright_black());
    let send_cmd = format!("{} {}: {}", "/send".bold().bright_blue(), "<file>".bright_blue(), "Send a file to the other user".bright_black());

    return format!("--------------------\n{}\n{}\n{}\n-----------------", rec_cmd, name_cmd, send_cmd);
}


pub async fn on_command(line: &str) -> anyhow::Result<()> {
    if is_command(line, vec!["rec", "receiver", "r"]) {
        return on_receiver(line).await;
    } else if is_command(line, vec!["n", "name"]) {
        return on_name(line).await;
    } else if is_command(line, vec!["s", "send"]) {
        return on_send(line).await;
    } else if is_command(line, vec!["h", "help"]) {
        println!("{}", get_help_str());
    } else {
        println!("{}", "Unrecognized command. Use /help for commands".red());
    }

    Ok(())
}
