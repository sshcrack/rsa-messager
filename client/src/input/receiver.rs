use std::str::FromStr;

use crate::util::{consts::{BASE_URL, CURR_ID}};
use anyhow::anyhow;
use inquire::Select;
use uuid::Uuid;

pub async fn select_receiver() -> anyhow::Result<Uuid> {
    let state = BASE_URL.read().await;

    let base = state.to_string();
    drop(state);

    let list_url = format!("http://{}/list", base);

    println!("Fetching available clients {}...", list_url);
    let client = reqwest::Client::new();
    let resp = client.get(list_url.to_string()).send().await;

    if resp.is_err() {
        eprintln!("Could not fetch from {}", list_url);
        return Err(anyhow!(resp.unwrap_err()));
    }

    let resp = resp.unwrap();
    let text = resp.text().await?;

    let mut available: Vec<String> = serde_json::from_str(&text)?;
    let mut found_index = 9999;

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

    if found_index != 9999 {
        available[found_index] = format!("{} (you)", available[found_index]);
    }

    let mut select_prompt = Select::new("Receiver:", available);
    if found_index != 9999 {
        select_prompt.starting_cursor = found_index;
    }

    let selected = select_prompt.prompt()?;
    let selected = selected.replace(" (you)", "");
    let selected = Uuid::from_str(&selected)?;

    return Ok(selected);
}
