use std::{collections::HashMap, fmt::Write, sync::Arc};

use anyhow::anyhow;
use colored::Colorize;
use futures_util::future::select_all;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::file::{processing::tools::get_max_chunks, types::FileInfo};
use tokio::{
    sync::{
        mpsc::{self, UnboundedSender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use uuid::Uuid;

use crate::{
    file::tools::{get_hash_progress, WorkerProgress},
    util::{arcs::get_concurrent_threads, tools::get_avg},
};

use super::worker::DownloadWorker;

type WorkersType = Arc<RwLock<Vec<DownloadWorker>>>;

type WorkerTx = UnboundedSender<WorkerProgress>;
type ArcWorkerTx = Arc<RwLock<WorkerTx>>;

type WorkerRx = UnboundedReceiverStream<WorkerProgress>;
type ArcWorkerRx = Arc<RwLock<WorkerRx>>;

type UpdateThread = Arc<Mutex<Option<JoinHandle<()>>>>;
type FileLockType = Arc<Mutex<bool>>;

type ProgressMap = HashMap<u64, f32>;
type ProgressType = Arc<RwLock<ProgressMap>>;

#[derive(Debug)]
pub struct Downloader {
    info: FileInfo,
    uuid: Uuid,

    threads: Option<u64>,

    workers: WorkersType,
    progress: ProgressType,

    sender_pubkey: Rsa<Public>,

    update_thread: UpdateThread,

    file_lock: FileLockType,

    worker_tx: ArcWorkerTx,
    worker_rx: ArcWorkerRx,
}

impl Downloader {
    pub fn new(uuid: &Uuid, sender_pubkey: Rsa<Public>, info: &FileInfo) -> Self {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();

        let worker_rx = UnboundedReceiverStream::new(worker_rx);
        return Self {
            uuid: uuid.clone(),
            info: info.clone(),
            threads: None,
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            sender_pubkey,
            update_thread: Arc::new(Mutex::new(None)),
            file_lock: Arc::new(Mutex::new(false)),
            worker_tx: Arc::new(RwLock::new(worker_tx)),
            worker_rx: Arc::new(RwLock::new(worker_rx)),
        };
    }

    pub async fn initialize(&mut self, max_chunks: u64) -> anyhow::Result<()> {
        let handle = self.listen_for_progress_updates()?;

        let mut state = self.update_thread.lock().await;
        *state = Some(handle);

        drop(state);

        let info = self.info.clone();
        let workers = self.workers.clone();
        let uuid = self.uuid.clone();
        let sender_pubkey = self.sender_pubkey.clone();
        let file_lock = self.file_lock.clone();
        let worker_tx = self.worker_tx.clone();

        let to_spawn = get_concurrent_threads().await.min(max_chunks);
        self.threads = Some(to_spawn);

        if info.path.is_none() {
            return Err(anyhow!(
                "Can not initialize downloader when download path is none."
            ));
        }

        trace!("Waiting for read...");
        let state = workers.read().await;

        if !state.is_empty() {
            drop(state);
            return Err(anyhow!("Workers already spawned."));
        }
        drop(state);

        trace!("Spawning {} workers", to_spawn);
        let mut state = workers.write().await;
        for i in 0..to_spawn {
            let worker = DownloadWorker::new(
                i,
                uuid,
                sender_pubkey.clone(),
                info.clone(),
                file_lock.clone(),
                worker_tx.clone(),
            )?;
            state.push(worker);
        }

        drop(state);
        Ok(())
    }

    pub async fn start_downloading(&self, chunk: u64) -> anyhow::Result<()> {
        let prog = self.progress.read().await;
        if prog.contains_key(&chunk) {
            warn!("Already downloading chunk {}", chunk);

            drop(prog);
            return Ok(());
        }

        drop(prog);

        trace!("Waiting to download chunk {}", chunk);
        let mut state = self.workers.write().await;
        let futures = state.iter_mut().map(|e| {
            Box::pin(async {
                let res = e.wait_for_end().await;
                if res.is_err() {
                    eprintln!("Error when waiting for end: {}", res.unwrap_err());
                    return None;
                }
                return Some(e.get_working_id());
            })
        });

        let selected = select_all(futures);
        let (available_worker_id, ..) = selected.await;
        //TODO maybe abort other futures?

        if available_worker_id.is_none() {
            return Err(anyhow!("Available Worker is none."));
        }

        let available_worker_id = available_worker_id.unwrap();
        let available_worker_id = usize::try_from(available_worker_id)?;

        let worker = state.get_mut(available_worker_id);
        if worker.is_none() {
            eprint!("Unknown Error occurred while getting worker for new downloader.");
            return Err(anyhow!(
                "Unknown Error in function start download (worker none)"
            ));
        }

        let mut prog = self.progress.write().await;
        trace!("Starting worker with id {}", chunk);
        let worker = worker.unwrap();
        let res = worker.start(chunk);
        drop(state);

        prog.insert(chunk, 0.0);
        drop(prog);

        res?;
        Ok(())
    }

    fn listen_for_progress_updates(&self) -> anyhow::Result<JoinHandle<()>> {
        let prog_arc = self.progress.clone();
        let file_arc = Arc::new(RwLock::new(self.info.clone()));
        let max_size = self.info.size;
        let worker_rx_arc = self.worker_rx.clone();

        let e = tokio::spawn(async move {
            let pb = ProgressBar::new(max_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            let mut worker_rx = worker_rx_arc.write().await;
            while let Some(el) = worker_rx.next().await {
                let WorkerProgress { chunk, progress } = el;

                let mut state = prog_arc.write().await;

                let old_prog = state.get(&chunk).unwrap_or(&(0 as f32)).to_owned();
                state.insert(chunk, progress);
                let mut downloader_done = false;
                if progress >= 1.0 && old_prog < 1.0 as f32 {
                    trace!("Downloader worker {} finished.", chunk);
                    let e = Downloader::on_worker_done(&state, &file_arc, &pb).await;
                    if e.is_err() {
                        let err = e.unwrap_err();
                        println!(
                            "{}",
                            format!("Could not send on worker_done update: {}", err).red()
                        );
                    } else {
                        downloader_done = e.unwrap();
                    }
                }

                if !downloader_done {
                    Downloader::print_update(&pb, &state, max_size);
                }
                drop(state);
            }
            drop(worker_rx);
        });

        return Ok(e);
    }

    fn print_update(pb: &ProgressBar, progress: &ProgressMap, max_size: u64) {
        let mut percentages: Vec<f32> = progress.iter().map(|e| e.1.clone()).collect();
        let max_chunks = get_max_chunks(max_size);

        let left = max_chunks - progress.len() as u64;
        for _ in 0..left {
            percentages.push(0 as f32);
        }

        let avg = get_avg(&percentages);
        if avg.is_err() {
            warn!("Could not get avg: {}", avg.unwrap_err());
            return;
        }

        let avg = avg.unwrap();
        let curr = std::cmp::min(max_size, (avg * (max_size as f32)).floor() as u64);

        pb.set_position(curr);
    }

    async fn get_chunks_completed(map: &ProgressMap) -> usize {
        let done: Vec<&f32> = map
            .values()
            .filter(|e| e.to_owned() >= &(1.0 as f32))
            .collect();
        return done.len();
    }

    async fn get_hash_progress(file: &FileInfo) -> anyhow::Result<Vec<u8>> {
        let path = file.path.clone();
        if path.is_none() {
            return Err(anyhow!(
                "Could not check for hash because local path is none."
            ));
        }

        let path = path.unwrap();
        let path = path.to_str().unwrap();

        let hash = get_hash_progress(path.to_owned()).await?;

        return Ok(hash);
    }

    async fn on_worker_done(
        map: &ProgressMap,
        file_arc: &Arc<RwLock<FileInfo>>,
        pb: &ProgressBar,
    ) -> anyhow::Result<bool> {
        let s = file_arc.read().await;
        let max_chunks = get_max_chunks(s.size) as usize;
        let curr_completed = Downloader::get_chunks_completed(map).await;

        if curr_completed < max_chunks {
            drop(s);
            return Ok(false);
        }

        pb.finish_and_clear();
        println!(
            "{}",
            format!("Calculating hash for downloaded file...").yellow()
        );
        let curr_hash = Downloader::get_hash_progress(&s).await?;

        let expected = (&s.hash).clone();

        let is_valid = curr_hash == expected;
        if is_valid {
            println!("{}",
                    format!("Hashes {} and {} match.", hex::encode(expected), hex::encode(curr_hash))
                    .green()
            );
            println!(
                "{}",
                format!(
                    "File '{}' has been downloaded successfully.",
                    s.filename.yellow()
                )
                .green()
            );
        } else {
            println!(
                "{}",
                format!(
                    "Could not download file as hashes did not match (expected {} got {})",
                    hex::encode(expected),
                    hex::encode(curr_hash)
                )
                .red()
            )
        }

        drop(s);
        return Ok(true);
    }
}
