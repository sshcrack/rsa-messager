use std::{str::FromStr, sync::atomic::Ordering};

use colored::Colorize;
use uuid::Uuid;

use crate::{
    input::receiver::select_receiver,
    util::{consts::{CURR_ID, RECEIVER, SEND_DISABLED}},
};

pub async fn on_uid(
    rec: &str
) -> anyhow::Result<()> {
    let mut state = CURR_ID.write().await;
    let uuid = Uuid::from_str(rec)?;

    *state = Some(uuid.clone());

    drop(state);

    println!("Current id set to {}", rec);
    let e = select_receiver().await?;
    let mut state = RECEIVER.write().await;
    *state = Some(e);

    drop(state);

    SEND_DISABLED.store(false, Ordering::Relaxed);
    let e = "Chatroom is now open!".to_string().on_green();

    println!("{}", e);
    Ok(())
}
