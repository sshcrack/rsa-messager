use anyhow::anyhow;
use uuid::Uuid;

use crate::util::{consts::BASE_URL, types::UserInfoBasic};

pub async fn get_user_info(uuid: &Uuid) -> anyhow::Result<UserInfoBasic> {
    let state = BASE_URL.read().await;

    let base = state.to_string();
    drop(state);

    let info_url = format!("http://{}/info?id={}", base, uuid.to_string());

    let client = reqwest::Client::new();
    let resp = client.get(info_url.to_string()).send().await;

    if resp.is_err() {
        eprintln!("Could not fetch from {}", info_url);
        return Err(anyhow!(resp.unwrap_err()));
    }

    let resp = resp.unwrap();
    let text = resp.text().await?;
    let json: UserInfoBasic = serde_json::from_str(&text)?;

    return Ok(json);
}
