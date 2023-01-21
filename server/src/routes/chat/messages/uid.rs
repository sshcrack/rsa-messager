use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::modes::Modes;

pub fn on_uid(my_id: &Uuid, tx: &UnboundedSender<Message>) {

    let my_id_b = my_id.to_string().as_bytes().to_vec();
    let packet = Modes::UidReply.get_send(&my_id_b);

    let res = tx.send(Message::binary(packet));
    if res.is_err() {
        eprintln!("{}", res.unwrap_err());
    }
}
