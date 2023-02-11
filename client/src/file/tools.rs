use std::fmt::Write;

use anyhow::anyhow;
use bytes::Buf;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use openssl::hash::Hasher;
use packets::{consts::MSG_DIGEST, file::types::FileInfo};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use uuid::Uuid;

use crate::util::consts::PENDING_FILES;

pub async fn get_pending_file(uuid: Uuid) -> anyhow::Result<FileInfo> {
    let state = PENDING_FILES.read().await;
    let temp = state.get(&uuid);
    if temp.is_none() {
        return Err(anyhow!("Could not find pending upload"));
    }

    return Ok(temp.unwrap().to_owned());
}

pub async fn get_hash_progress(file_path: String) -> anyhow::Result<Vec<u8>> {
    let file = File::open(file_path).await?;
    let size = file.metadata().await?.len();

    let mut reader = FramedRead::new(file, BytesCodec::new());
    let mut hasher = Hasher::new(*MSG_DIGEST)?;


    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    .unwrap()
    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    while let Some(chunk) = reader.next().await {
        let chunk = chunk?;
        let chunk = chunk.chunk();

        pb.inc(chunk.len().try_into()?);
        hasher.update(chunk)?;
    }

    pb.finish();
    return Ok(hasher.finish()?.to_vec());
}

#[derive(Debug, Clone)]
pub struct WorkerProgress {
    pub chunk: u64,
    pub progress: f32,
}
