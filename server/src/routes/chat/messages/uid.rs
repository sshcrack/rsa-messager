use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{modes::Modes, types::TXChannel, tools::send_msg};

pub fn on_uid(my_id: &Uuid, tx: &TXChannel) -> anyhow::Result<()> {
    let my_id_b = my_id.as_bytes().to_vec();
    let packet = Modes::UidReply.get_send(&my_id_b);

    send_msg(tx, Message::binary(packet))?;
    Ok(())
}
