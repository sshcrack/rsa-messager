use log::trace;
use packets::{file::{types::FileInfo, processing::{start::FileStartProcessing, tools::get_max_threads}}, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::{tools::send_msg_specific, types::Users};

#[readonly::make]
pub struct Controller {
    #[readonly]
    pub file: FileInfo,

    uuid: Uuid,
    users: Users

}

impl Controller {
    pub async fn new(id: &Uuid, users: &Users, file: FileInfo) -> anyhow::Result<Self> {
        let threads = get_max_threads(file.size);
        let id = id.clone();
        let users = users.clone();

        trace!("Sending start processing packet...");
        let to_send = FileStartProcessing {
            uuid: id,
            threads
        }.serialize();

        send_msg_specific(file.sender, &users, Message::binary(to_send)).await?;

        trace!("Returning");
        return Ok(Controller {
            file,
            uuid: id,
            users
        });
    }
}