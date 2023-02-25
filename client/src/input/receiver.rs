use std::str::FromStr;

use crate::{util::{consts::{BASE_URL, CURR_ID, CHAT_SYMM_KEYS}, msg::send_msg}, web::prefix::get_web_protocol};
use anyhow::anyhow;
use colored::Colorize;
use inquire::Select;
use packets::{communication::key_request::WantSymmKeyMsg, types::ByteMessage};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

const NOT_FOUND_ID: usize = 9999;

pub async fn select_receiver() -> anyhow::Result<Uuid> {
    let res = tokio::spawn(async move {
        let state = BASE_URL.read().await;

        let base = state.to_string();
        drop(state);

        let protocol = get_web_protocol().await;
        let list_url = format!("{}//{}/list", protocol, base);

        println!("{}", format!("Fetching available clients {}...", list_url.purple()).bright_black());
        let resp = surf::get(list_url.to_string()).send().await;

        if resp.is_err() {
            eprintln!("Could not fetch from {}", list_url);
            return Err(anyhow!(resp.unwrap_err()));
        }

        let mut resp = resp.unwrap();
        let text = resp.body_string().await;
        if text.is_err() {
            return Err(text.unwrap_err().into_inner());
        }

        let text = text.unwrap();
        println!("Body Str is {}", text);
        let mut available: Vec<String> = text.split(",").map(|e| e.to_owned()).collect();
        let mut found_index = NOT_FOUND_ID;

        let state = CURR_ID.read().await;
        if state.is_some() {
            let mut i = 0;
            let id = state.as_ref().unwrap();
            let id = id.to_string();

            for el in available.clone() {
                if el.eq(&id) {
                    found_index = i;
                }

                i += 1;
            }
        }

        drop(state);

        if found_index != NOT_FOUND_ID {
            available[found_index] = format!("{} (you)", available[found_index]);
        }

        let mut select_prompt = Select::new("Receiver:", available);
        if found_index != NOT_FOUND_ID {
            select_prompt.starting_cursor = found_index;
        }

        let selected = select_prompt.prompt()?;
        let selected = selected.replace(" (you)", "");
        let selected = Uuid::from_str(&selected)?;

        return Ok(selected);
    });

    let rec = res.await??;
    send_msg(Message::Binary(
        WantSymmKeyMsg {
            user: rec
        }.serialize()
    )).await?;

    CHAT_SYMM_KEYS.write().await.insert(rec.clone(), None);
    return Ok(rec);
}
