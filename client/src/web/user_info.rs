use anyhow::anyhow;
use log::trace;
use uuid::Uuid;

use crate::{
    util::{arcs::get_base_url, types::UserInfoBasic},
    web::prefix::get_web_protocol,
};

pub async fn get_user_info(uuid: &Uuid) -> anyhow::Result<UserInfoBasic> {
    let uuid = uuid.clone();
    let e = tokio::spawn(async move {
        let protocol = get_web_protocol().await;
        let base = get_base_url().await;

        let info_url = format!("{}//{}/info?id={}", protocol, base, uuid.to_string());

        trace!("Requesting user info from {}...", info_url);
        let resp = surf::get(info_url.to_string()).await;

        if resp.is_err() {
            eprintln!("Could not fetch from {}", info_url);
            return Err(anyhow!(resp.unwrap_err()));
        }

        let mut resp = resp.unwrap();
        trace!("Resp parse text");
        let text = resp.body_string().await;
        if text.is_err() {
            return Err(text.unwrap_err().into_inner());
        }

        let text = text.unwrap();
        let json: UserInfoBasic = serde_json::from_str(&text)?;

        trace!("Done.");
        return Ok(json);
    });

    return e.await?;
}
