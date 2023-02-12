use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::anyhow;
use colored::Colorize;
use indicatif::HumanBytes;
use log::{debug, trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::{
    consts::CHUNK_SIZE_I64,
    encryption::sign::get_signature,
    file::{
        chunk::index::ChunkMsg,
        processing::{downloaded::ChunkDownloadedMsg, tools::get_max_chunks},
        types::FileInfo,
    },
    types::ByteMessage,
};
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{Mutex, RwLock, mpsc::UnboundedSender},
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    util::{
        arcs::{get_base_url, get_curr_keypair},
        msg::send_msg, consts::MAX_RETRIES
    },
    web::{prefix::get_web_protocol, progress::download_file}, file::tools::WorkerProgress,
};

pub type ProgressTX = UnboundedSender<WorkerProgress>;
pub type ArcProgressTX = Arc<RwLock<ProgressTX>>;

#[derive(Debug)]
pub struct DownloadWorker {
    worker_id: u64,
    uuid: Uuid,
    file: FileInfo,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
    running: bool,
    tx: ArcProgressTX,
    sender_key: Rsa<Public>,
    file_lock: Arc<Mutex<bool>>,
    aborted: Arc<RwLock<bool>>
}

impl DownloadWorker {
    pub fn new(
        worker_id: u64,
        uuid: Uuid,
        sender_key: Rsa<Public>,
        file: FileInfo,
        file_lock: Arc<Mutex<bool>>,
        progress_channel: ArcProgressTX,
    ) -> anyhow::Result<Self> {
        let FileInfo { size, path, .. } = file.clone();
        if path.is_none() {
            return Err(anyhow!("Invalid path, path is none. Downloader."));
        }

        let path = path.unwrap();
        let path = Path::new(&path);

        let curr_dir = std::env::current_dir()?;
        let curr_dir_path = curr_dir.clone();
        let curr_dir_path = curr_dir_path.as_path();

        let dir_path = path.parent().unwrap_or(curr_dir_path);
        let mut dir = dir_path.to_path_buf();

        if !dir.is_absolute() {
            let mut buf = PathBuf::new();
            buf.push(curr_dir);
            buf.push(dir.clone());

            dir = buf;
        }

        let left = fs2::available_space(dir)?;

        if left < size {
            eprintln!(
                "Not enough size on your disk left ({} left, {} needed)",
                HumanBytes(left),
                HumanBytes(file.size)
            );
            return Err(anyhow!("Size of file does not match with metadata"));
        }

        return Ok(DownloadWorker {
            worker_id,
            file,
            uuid,
            thread: None,
            tx: progress_channel,
            running: false,
            sender_key,
            file_lock,
            aborted: Arc::new(RwLock::new(false))
        });
    }

    async fn spawn_thread(&self, chunk_index: u64) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        if *self.aborted.read().await {
            return Err(anyhow!("Cannot start worker as it has been aborted."))
        }

        trace!(
            "Spawning new worker with i: {} uuid: {}",
            chunk_index,
            self.uuid
        );

        let file = self.file.clone();
        let tx = self.tx.clone();
        let uuid = self.uuid.clone();

        let i = chunk_index;

        let out_path = file.path.clone().unwrap();
        let size = file.size;
        let sender_key = self.sender_key.clone();
        let file_lock_arc = self.file_lock.clone();

        let handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let to_run = || async {
                let max_threads = get_max_chunks(size);

                if max_threads <= 0 {
                    warn!("Max Threads is 0 in index {}", i);
                    return Err(anyhow!("MaxThreads is zero"));
                }

                trace!(
                    "While loop with i: {} and max_threads {}...",
                    i,
                    max_threads
                );

                let file_lock = file_lock_arc.lock().await;
                let keypair = get_curr_keypair().await?;

                let uuid_signature = get_signature(&uuid.as_bytes().to_vec(), &keypair)?;

                let base_url = get_base_url().await;
                let http_protocol = get_web_protocol().await;

                let url = format!(
                    "{}//{}/file/download?index={}&uuid={}&signature={}",
                    http_protocol,
                    base_url,
                    i,
                    uuid.to_string(),
                    hex::encode(uuid_signature)
                );

                let tx_state = tx.read().await;
                let response = download_file(url, &tx_state, i).await;
                drop(tx_state);

                let response = response?;
                // Signature is validated in deserialize, so its fine
                let deserialized = ChunkMsg::deserialize(&response, &sender_key, &keypair);
                if deserialized.is_err() {
                    let e = deserialized.unwrap_err();
                    eprintln!("Deserialize err: {:?}", e);
                    return Err(e);
                }
                let deserialized = deserialized.unwrap();
                let encrypted = &deserialized.encrypted;

                let decrypted = deserialized.key.decrypt(encrypted)?;

                let offset = CHUNK_SIZE_I64 * i as i64;

                let path = Path::new(&out_path);
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await?;

                f.seek(SeekFrom::Current(offset)).await?;
                f.write_all(&decrypted).await?;

                drop(file_lock);

                send_msg(Message::Binary(
                    ChunkDownloadedMsg {
                        chunk_index: i,
                        uuid,
                    }
                    .serialize(),
                ))
                .await?;
                debug!("Worker {} of file {} done.", i, uuid);
                Ok(())
            };

            let mut retry_count = 0 as u64;
            let mut res;
            loop {
                res = to_run().await;
                if res.is_ok() || retry_count >= MAX_RETRIES {
                    break;
                }

                retry_count += 1;
                eprintln!("{}", format!("An error occurred while running a worker. Retrying ({} / {}).", retry_count, MAX_RETRIES).on_yellow());
            }
            drop(tx);

            if res.is_err() {
                let err = res.unwrap_err();
                eprintln!("Downloader Worker error: {:?}", err);
                return Err(err)
            }
            Ok(())
        });

        return Ok(handle);
    }

    pub async fn start(&mut self, chunk_index: u64) -> anyhow::Result<()> {
        if self.thread.is_some() {
            trace!(
                "Could not start thread on index {}. Already running.",
                chunk_index
            );
            return Err(anyhow!(format!(
                "Could not start new thread. Already running. Index: {}",
                chunk_index
            )));
        }

        self.running = true;
        let thread = self.spawn_thread(chunk_index).await?;
        self.thread = Some(thread);

        Ok(())
    }

    pub async fn wait_for_end(&mut self) -> anyhow::Result<()> {
        if self.thread.is_none() {
            return Ok(());
        }

        let res = self.thread.take().unwrap();

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

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        return self.running;
    }

    pub fn get_working_id(&self) -> u64 {
        return self.worker_id;
    }
}
