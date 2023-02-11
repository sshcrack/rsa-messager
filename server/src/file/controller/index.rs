use log::trace;
use packets::{file::{types::FileInfo, processing::start::FileStartProcessing}, types::ByteMessage};
use uuid::Uuid;
use warp::ws::Message;

use crate::utils::tools::send_msg_specific;

#[readonly::make]
pub struct Controller {
    #[readonly]
    pub file: FileInfo,

    uuid: Uuid
}

impl Controller {
    pub async fn new(id: &Uuid, file: FileInfo) -> anyhow::Result<Self> {
        //let threads = get_max_chunks(file.size);
        let id = id.clone();

        trace!("Sending start processing packet...");
        let to_send = FileStartProcessing {
            uuid: id,
            //threads
        }.serialize();

        send_msg_specific(file.sender, Message::binary(to_send)).await?;

        trace!("Returning");
        return Ok(Controller {
            file,
            uuid: id
        });
    }
}