use std::{io::SeekFrom, path::{Path, PathBuf}, sync::Arc};

use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::{
    consts::CHUNK_SIZE_I64,
    file::{
        chunk::index::{ChunkByteMessage, ChunkMsg},
        processing::{downloaded::ChunkDownloadedMsg, tools::get_max_threads},
        types::FileInfo,
    }, types::ByteMessage, encryption::sign::get_signature,
};
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    encryption::rsa::decrypt,
    util::{
        arcs::{get_base_url, get_curr_keypair},
        msg::send_msg,
    },
    web::{prefix::get_web_protocol, progress::download_file},
};

pub type ProgressChannel = Receiver<f32>;
pub type ArcProgressChannel = Arc<RwLock<ProgressChannel>>;

pub type ProgressTX = Sender<f32>;
pub type ArcProgressTX = Arc<RwLock<ProgressTX>>;

#[derive(Debug)]
pub struct DownloadWorker {
    worker_id: u64,
    uuid: Uuid,
    file: FileInfo,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
    running: bool,
    tx: ArcProgressTX,
    pub progress_channel: ArcProgressChannel,
    sender_key: Rsa<Public>,
    file_lock: Arc<Mutex<bool>>,
}

impl DownloadWorker {
    pub fn new(
        worker_id: u64,
        uuid: Uuid,
        sender_key: Rsa<Public>,
        file: FileInfo,
        file_lock: Arc<Mutex<bool>>,
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

        trace!("Getting available space at {}...", dir.to_str().unwrap());
        let left = fs2::available_space(dir)?;

        trace!("Left is {} size of file is {}", left, size);
        if left < size {
            eprintln!(
                "Not enough size on your disk left ({} left, {} needed)",
                pretty_bytes::converter::convert(left as f64),
                pretty_bytes::converter::convert(file.size as f64)
            );
            return Err(anyhow!("Size of file does not match with metadata"));
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let arc = Arc::new(RwLock::new(rx));
        let arc_tx = Arc::new(RwLock::new(tx));

        return Ok(DownloadWorker {
            worker_id,
            file,
            uuid,
            thread: None,
            tx: arc_tx,
            progress_channel: arc,
            running: false,
            sender_key,
            file_lock,
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

        let out_path = file.path.clone().unwrap();
        let size = file.size;
        let sender_key = self.sender_key.clone();
        let file_lock_arc = self.file_lock.clone();

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

                let file_lock = file_lock_arc.lock().await;
                let keypair = get_curr_keypair().await?;


                let uuid_signature = get_signature(&uuid.as_bytes().to_vec(), &keypair)?;

                let base_url = get_base_url().await;
                let http_protocol = get_web_protocol().await;

                let url = format!(
                    "{}//{}/file/download?i={}&uuid={}&signature={}",
                    http_protocol,
                    base_url,
                    i,
                    uuid.to_string(),
                    hex::encode(uuid_signature)
                );

                let client = reqwest::Client::new();
                let response = download_file(&client, url, &tx).await?;

                // Signature is validated in deserialize, so its fine
                let deserialized = ChunkMsg::deserialize(&response, &sender_key)?;
                let encrypted = &deserialized.encrypted;

                let decrypted = decrypt(&keypair, encrypted)?;

                let offset = CHUNK_SIZE_I64 * i as i64;

                trace!("Chunk downloaded. Saving...");
                let path = Path::new(&out_path);
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await?;

                f.seek(SeekFrom::Current(offset)).await?;
                f.write_all(&decrypted).await?;

                drop(file_lock);

                tx.send(1 as f32)?;

                send_msg(Message::Binary(
                    ChunkDownloadedMsg {
                        chunk_index: i,
                        uuid,
                    }
                    .serialize(),
                )).await?;
                debug!("Worker {} of file {} done.", i, uuid);
                Ok(())
            };

            let res = to_run().await;
            drop(tx);

            if res.is_err() {
                eprintln!("Downloader Worker error:");
                res?;
            }
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
