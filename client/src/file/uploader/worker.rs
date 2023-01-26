use std::{io::SeekFrom, path::Path, sync::Arc};

use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::{
    consts::{CHUNK_SIZE, ONE_MB_SIZE},
    file::{processing::tools::get_max_threads, types::FileInfo, chunk::index::{ChunkByteMessage, ChunkMsg}}, encryption::sign::get_signature,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, BufReader},
    sync::RwLock,
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{encryption::rsa::encrypt, util::arcs::{get_curr_keypair, get_base_url}, web::{prefix::get_web_protocol, progress::upload_file}};

pub type ProgressChannel = Receiver<f32>;
pub type ArcProgressChannel = Arc<RwLock<ProgressChannel>>;

pub type ProgressTX = Sender<f32>;
pub type ArcProgressTX = Arc<RwLock<ProgressTX>>;

#[derive(Debug)]
pub struct UploadWorker {
    worker_id: u64,
    uuid: Uuid,
    file: FileInfo,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
    running: bool,
    tx: ArcProgressTX,
    pub progress_channel: ArcProgressChannel,
    key: Rsa<Public>,
}

impl UploadWorker {
    pub fn new(
        worker_id: u64,
        uuid: Uuid,
        key: Rsa<Public>,
        file: FileInfo,
    ) -> anyhow::Result<UploadWorker> {
        let FileInfo { filename, size, .. } = file.clone();

        let path = Path::new(&filename);

        if !path.is_file() {
            eprintln!("Could not send file at {} (does not exist)", filename);
            return Err(anyhow!("File {} does not exist.", filename));
        }

        let metadata = path.metadata()?;
        if metadata.len() != size {
            eprintln!(
                "Size of file does not match with metadata (metadata {}, given {})",
                metadata.len(),
                file.size
            );
            return Err(anyhow!("Size of file does not match with metadata"));
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let arc = Arc::new(RwLock::new(rx));
        let arc_tx = Arc::new(RwLock::new(tx));

        return Ok(UploadWorker {
            worker_id,
            file,
            uuid,
            thread: None,
            tx: arc_tx,
            progress_channel: arc,
            running: false,
            key,
        });
    }

    fn spawn_thread(&self, thread_index: u64) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        trace!(
            "Spawning new worker with i: {} uuid: {}",
            thread_index,
            self.uuid
        );

        let file = self.file.clone();
        let tx = self.tx.clone();
        let uuid = self.uuid.clone();

        let i = thread_index;

        let filename = file.filename.clone();
        let size = file.size;
        let key = self.key.clone();

        let handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let tx = tx.read().await;
            let to_run = || async {
                let max_threads = get_max_threads(size);

                if max_threads <= 0 {
                    warn!("Max Threads is 0 in index {}", i);
                    return Err(anyhow!("MaxThreads is zero"));
                }

                trace!(
                    "While loop with i: {} and max_threads {}...",
                    i,
                    max_threads
                );

                let path = Path::new(&filename);
                let f = File::open(&path).await?;
                let mut buf = BufReader::new(f);
                let seek_to = i64::try_from(CHUNK_SIZE * i)?;

                buf.seek(SeekFrom::Current(seek_to)).await?;

                let is_last_chunk = (i + max_threads) >= max_threads;

                let chunk_size_u64 = if is_last_chunk {
                    std::cmp::min(CHUNK_SIZE, size)
                } else {
                    CHUNK_SIZE
                };
                let chunk_size = usize::try_from(chunk_size_u64)?;

                let mut chunk = Vec::with_capacity(chunk_size);

                trace!("Reading file with chunk size {} (i: {})", chunk_size, i);
                let mut bytes_read = 0;
                while bytes_read < chunk_size {
                    let to_read = std::cmp::min(ONE_MB_SIZE, chunk_size_u64);
                    let to_read = usize::try_from(to_read)?;
                    let mut small_chunk = Vec::with_capacity(to_read);

                    buf.read_exact(&mut small_chunk).await?;
                    chunk.append(&mut small_chunk);

                    let percentage = (bytes_read as f32) / (chunk_size as f32) * 0.5;
                    tx.send(percentage)?;

                    bytes_read += to_read;
                }

                let encrypted = encrypt(key, &buf.buffer().to_vec())?;
                let keypair = get_curr_keypair().await?;
                let signature = get_signature(&encrypted, &keypair)?;

                let base_url = get_base_url().await;
                let http_protocol = get_web_protocol().await;

                let url = format!("{}//{}/file/upload", http_protocol, base_url);
                let client = reqwest::Client::new();

                trace!("Uploading chunk {} to {}...", i, url);
                let body = ChunkMsg {
                    signature,
                    encrypted,
                    uuid,
                    chunk_index: i
                }.serialize();

                trace!("Uploading chunk {}...", i);

                upload_file(&client, url, body).await?;
                tx.send(1 as f32)?;
                debug!("Worker {} of file {} done.", i, uuid);
                Ok(())
            };

            let res = to_run().await;
            drop(tx);

            res?;
            Ok(())
        });

        return Ok(handle);
    }

    pub fn start(&mut self, thread_index: u64) -> anyhow::Result<()> {
        if self.thread.is_some() {
            trace!(
                "Could not start thread on index {}. Already running.",
                thread_index
            );
            return Err(anyhow!(format!(
                "Could not start new thread. Already running. Index: {}",
                thread_index
            )));
        }

        self.running = true;
        let thread = self.spawn_thread(thread_index)?;
        self.thread = Some(thread);

        Ok(())
    }

    pub async fn wait_for_end(&mut self) -> anyhow::Result<()> {
        if self.thread.is_none() {
            warn!("Thread is none. Could not wait for end so returning instantly.");
            return Ok(());
        }

        let res = self.thread.take().unwrap();
        tokio::task::yield_now().await;

        let e = res.await;
        if e.is_err() {
            trace!("Checking for join err:");
        }
        let e = e?;
        if e.is_err() {
            trace!("Checking for runtime error:");
        }
        e?;

        self.running = false;

        return Ok(());
    }

    pub fn is_running(&self) -> bool {
        return self.running;
    }

    pub fn get_working_id(&self) -> u64 {
        return self.worker_id;
    }
}
