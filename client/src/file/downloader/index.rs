use std::{collections::HashMap, fmt::Write, ops::Add, sync::Arc};

use anyhow::anyhow;
use colored::Colorize;
use crossbeam_channel::{Receiver, Sender};
use futures_util::future::select_all;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::file::{processing::tools::get_max_chunks, types::FileInfo};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{util::{consts::CONCURRENT_THREADS, tools::get_avg}, file::tools::WorkerProgress};

use super::worker::DownloadWorker;

type WorkersType = Arc<RwLock<Vec<DownloadWorker>>>;

type WorkerTx = Sender<WorkerProgress>;
type WorkerRx = Receiver<WorkerProgress>;

type UpdateThread = Arc<Mutex<Option<JoinHandle<()>>>>;
type FileLockType = Arc<Mutex<bool>>;
type ChunksCompletedType = Arc<RwLock<u64>>;

type ProgressMap = HashMap<u64, f32>;
type ProgressType = Arc<RwLock<ProgressMap>>;

#[derive(Debug)]
pub struct Downloader {
    info: FileInfo,
    uuid: Uuid,

    chunks_completed: ChunksCompletedType,
    threads: Option<u64>,

    workers: WorkersType,
    progress: ProgressType,

    sender_pubkey: Rsa<Public>,

    update_thread: UpdateThread,

    file_lock: FileLockType,

    worker_tx: WorkerTx,
    worker_rx: WorkerRx
}

impl Downloader {
    pub fn new(uuid: &Uuid, sender_pubkey: Rsa<Public>, info: &FileInfo) -> Self {
        let (worker_tx, worker_rx) = crossbeam_channel::unbounded();

        return Self {
            uuid: uuid.clone(),
            info: info.clone(),
            threads: None,
            chunks_completed: Arc::new(RwLock::new(0)),
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            sender_pubkey,
            update_thread: Arc::new(Mutex::new(None)),
            file_lock: Arc::new(Mutex::new(false)),
            worker_tx: worker_tx,
            worker_rx: worker_rx,
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

        let to_spawn = std::cmp::min(CONCURRENT_THREADS, max_chunks);
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
                    worker_tx.clone()
                )?;
                state.push(worker);
            }

            drop(state);
            Ok(())

    }

    pub async fn start_downloading(&self, chunk: u64) -> anyhow::Result<()> {
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

        trace!("Starting worker with id {}", chunk);
        let worker = worker.unwrap();
        let res = worker.start(chunk);
        drop(state);

        res?;
        Ok(())
    }

    fn listen_for_progress_updates(&self) -> anyhow::Result<JoinHandle<()>> {
        let prog_arc = self.progress.clone();
        let chunks_arc = self.chunks_completed.clone();
        let file_arc = Arc::new(RwLock::new(self.info.clone()));
        let max_size = self.info.size;
        let rec = self.worker_rx.clone();

        let e = tokio::spawn(async move {
            let pb = ProgressBar::new(max_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            println!("Listening for updates.");
            while let Ok(el) = rec.recv() {
                let WorkerProgress { chunk, progress } = el;

                let mut state = prog_arc.write().await;

                state.insert(chunk, progress);
                if progress >= 1.0 {
                    trace!("Downloader worker {} finished.", chunk);
                    Downloader::on_worker_done(&chunks_arc, &file_arc).await;
                }

                Downloader::print_update(&pb, &state, max_size);
                drop(state);
            }
        });

        return Ok(e);
    }


    fn print_update(pb: &ProgressBar, progress: &ProgressMap, max_size: u64) {
        let mut percentages: Vec<f32> = progress.iter().map(|e| e.1.clone()).collect();

        let left = max_size - progress.len() as u64;
        for _ in 0..left {
            percentages.push(0 as f32);
        }

        let avg = get_avg(&percentages);
        if avg.is_err() {
            warn!("Could not get avg: {}", avg.unwrap_err());
            return;
        }

        let avg = avg.unwrap();
        println!("Avg is {}", avg);
        let curr = std::cmp::min(max_size, (avg * (max_size as f32)).floor() as u64);

        pb.set_position(curr);
    }

    async fn on_worker_done(chunks_completed: &Arc<RwLock<u64>>, file_arc: &Arc<RwLock<FileInfo>>) {
        let mut s = chunks_completed.write().await;
        *s = s.add(1 as u64);

        let curr_completed = s.clone();
        drop(s);

        let s = file_arc.read().await;
        let max_chunks = get_max_chunks(s.size);
        if curr_completed >= max_chunks {
            println!(
                "{}",
                format!(
                    "File '{}' has been downloaded successfully.",
                    s.filename.yellow()
                )
                .green()
            );
        }

        drop(s);
    }
}
