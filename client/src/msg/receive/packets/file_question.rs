use colored::Colorize;
use tokio_tungstenite::tungstenite::Message;

use crate::util::{
    modes::Modes,
    msg::{get_input, send_msg},
    tools::{uuid_from_vec, uuid_to_name},
};

pub async fn on_file_question(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let sender = uuid_from_vec(data)?;
    let filename = String::from_utf8(data.to_owned())?;

    let sender_name = uuid_to_name(sender).await?;

    let confirm_msg = format!(
        "{} wants to send {}. Accept? (y/n)",
        sender_name.green(),
        filename.yellow()
    );
    println!("{}", confirm_msg);

    let accepted:bool;
    loop {
        let input = get_input().await?;
        println!("Received input {:#?}", input);
        if input.is_none() {
            eprintln!("You have to enter either y/n");
            continue;
        }

        let input = input.unwrap().to_lowercase();
        if !input.eq("y") && !input.eq("n") {
            eprintln!("You have to enter either y/n");
            continue;
        }

        println!("Received input {}", input);
        accepted = input.eq("y");
        break;
    }

    if !accepted {
        let denied = format!("You denied {} send request.", filename.bright_red());
        println!("{}", denied.red())
    } else {
        let allowed = format!("Receiving {}{}", filename.bright_green(), "...".yellow());
        println!("{}", allowed.green());
    }

    let accepted_vec: u8 = if accepted == true { 0 } else { 1 };
    let to_send = Modes::SendFileQuestionReply.get_send(&vec![accepted_vec]);

    send_msg(Message::binary(to_send)).await?;
    Ok(())
}
