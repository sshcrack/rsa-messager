use colored::Colorize;

use crate::util::tools::{uuid_from_vec, uuid_to_name};

pub async fn on_file_question_reply(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let sender = uuid_from_vec(data)?;
    let filename = String::from_utf8(data.to_owned())?;

    let sender_name = uuid_to_name(sender).await?;

    let confirm_msg = format!(
        "{} {} your file request of file '{}'.",
        sender_name.bright_blue(),
        "accepted".green(),
        filename.yellow()
    );
    println!("{}", confirm_msg);
    Ok(())
}
