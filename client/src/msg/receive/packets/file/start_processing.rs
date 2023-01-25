use colored::Colorize;
use log::trace;
use packets::{file::processing::start::FileStartProcessing, types::ByteMessage};

use crate::{util::{tools::uuid_to_name, consts::FILE_UPLOADS}, file::{tools::get_pending_file, uploader::index::Uploader}, encryption::rsa::get_pubkey_from_rec};

pub async fn on_start_processing(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileStartProcessing { threads, uuid } = FileStartProcessing::deserialize(data)?;

    let file = get_pending_file(uuid).await?;
    let receiver = file.receiver.clone();
    let receiver_name = uuid_to_name(receiver).await?;

    let plural = if threads > 1 { "s" } else { "" };

    println!("{}", format!("{} file '{}' to user '{}' ({} thread{})", "Starting to upload".green(), file.filename.yellow(), receiver_name.yellow(), threads, plural));

    let key = get_pubkey_from_rec(&receiver).await?;
    let mut state = FILE_UPLOADS.write().await;
    let mut uploader = Uploader::new(&uuid, key, &file);

    let res = uploader.start(threads).await;
    if res.is_err() {
        drop(state);
        trace!("Error occurred on start_processing 1");
        return res;
    }

    state.insert(uuid, uploader);
    drop(state);

    Ok(())
}
