use anyhow::anyhow;

use crate::{consts::BASE_URL, msg::types::UserInfoBasic};

pub async fn get_user_info(uuid: &str) -> anyhow::Result<UserInfoBasic>{
    let info_url = format!("http://{}/info?id={}", BASE_URL, uuid);

    let client = reqwest::Client::new();
    let resp = client.get(info_url.to_string())
        .send()
        .await;


    if resp.is_err() {
        eprintln!("Could not fetch from {}", info_url);
        return Err(anyhow!(resp.unwrap_err()));
    }

    let resp = resp.unwrap();
    let text = resp.text().await?;
    let json: UserInfoBasic = serde_json::from_str(&text)?;

    return Ok(json);
}