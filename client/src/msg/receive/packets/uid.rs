use std::sync::atomic::Ordering;

use colored::Colorize;
use packets::{initialize::uid_reply::UidReplyMsg, types::ByteMessage};

use crate::{
    input::receiver::select_receiver,
    util::consts::{RECEIVER, SEND_DISABLED, CURR_ID},
};

pub async fn on_uid(
    data: &mut Vec<u8>
) -> anyhow::Result<()> {
    let UidReplyMsg { uuid } = UidReplyMsg::deserialize(data)?;

    println!("Current id set to {}", uuid);
    let mut state = CURR_ID.write().await;
    *state = Some(uuid);

    drop(state);

    let e = select_receiver().await?;
    let mut state = RECEIVER.write().await;
    *state = Some(e);

    drop(state);

    SEND_DISABLED.store(false, Ordering::Relaxed);
    let e = "Chatroom is now open!".to_string().on_green();

    println!("{}", e);
    Ok(())
}
