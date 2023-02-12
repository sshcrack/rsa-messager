use anyhow::anyhow;
use log::trace;
use packets::other::info::UserInfoBasic;
use uuid::Uuid;

use crate::{
    util::{arcs::get_base_url},
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
        let bytes = resp.body_bytes().await;
        if bytes.is_err() {
            return Err(bytes.unwrap_err().into_inner());
        }

        let bytes = bytes.unwrap();
        let info: UserInfoBasic = UserInfoBasic::deserialize(&bytes)?;

        trace!("Done.");
        return Ok(info);
    });

    return e.await?;
}
