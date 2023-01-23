use colored::Colorize;
use packets::{file::processing::start::FileStartProcessing, types::WSMessage};

use crate::{util::{tools::uuid_to_name, consts::FILE_UPLOADS}, file::{tools::get_pending_upload, uploader::index::Uploader}};

pub async fn on_start_processing(data: &mut Vec<u8>) -> anyhow::Result<()> {
    let FileStartProcessing { threads, uuid } = FileStartProcessing::deserialize(data)?;

    let file = get_pending_upload(uuid).await?;
    let receiver = file.receiver.clone();
    let receiver_name = uuid_to_name(receiver).await?;

    let plural = if threads > 1 { "s" } else { "" };

    let mut state = FILE_UPLOADS.write().await;
    let mut uploader = Uploader::new(&uuid, &file);

    uploader.start(threads)?;
    state.insert(uuid, uploader);

    drop(state);

    println!("{}", format!("{} file '{}' to user '{}' ({} thread{})", "Starting to upload".green(), file.filename.yellow(), receiver_name.yellow(), threads, plural));
    Ok(())
}
