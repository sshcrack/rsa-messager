use std::{fmt::Write, time::Duration};

use anyhow::anyhow;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use openssl::hash::Hasher;
use packets::{consts::{MSG_DIGEST, ONE_MB_SIZE}, file::types::FileInfo};
use tokio::{fs::File, io::AsyncReadExt};
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
    let mut file = File::open(file_path).await?;
    let size = file.metadata().await?.len();

    let mut hasher = Hasher::new(*MSG_DIGEST)?;

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
    .unwrap()
    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));
    pb.enable_steady_tick(Duration::from_millis(250));

    let mut chunk;
    loop {
        chunk = Vec::with_capacity(ONE_MB_SIZE as usize);
        let e = file.read_buf(&mut chunk).await?;

        pb.inc(e.try_into().unwrap());
        hasher.update(&chunk)?;
        if e == 0 { break; }
    }

    pb.disable_steady_tick();
    pb.finish();
    return Ok(hasher.finish()?.to_vec());
}

#[derive(Debug, Clone)]
pub struct WorkerProgress {
    pub chunk: u64,
    pub progress: f32,
}
