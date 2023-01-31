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

use crate::util::{consts::CONCURRENT_THREADS, tools::get_avg};

use super::worker::DownloadWorker;

type WorkersType = Arc<RwLock<Vec<DownloadWorker>>>;
type ProgressType = Arc<RwLock<HashMap<u64, f32>>>;

type UpdateTx = Arc<RwLock<Sender<bool>>>;
type UpdateRx = Arc<RwLock<Receiver<bool>>>;
type UpdateThread = Arc<Mutex<Option<JoinHandle<()>>>>;
type FileLockType = Arc<Mutex<bool>>;
type ProgressBarType = Arc<RwLock<Option<ProgressBar>>>;
type ChunksCompletedType = Arc<RwLock<u64>>;

#[derive(Debug)]
pub struct Downloader {
    info: FileInfo,
    uuid: Uuid,

    chunks_completed: ChunksCompletedType,
    threads: Option<u64>,

    workers: WorkersType,
    progress: ProgressType,

    sender_pubkey: Rsa<Public>,

    update_tx: UpdateTx,
    update_rx: UpdateRx,
    update_thread: UpdateThread,

    // Bool is just a placeholder
    file_lock: FileLockType,
    progress_bar: ProgressBarType,
}

#[derive(Clone)]
struct ListenProgInfo {
    progress: ProgressType,
    update_tx: UpdateTx,
    chunks_completed: ChunksCompletedType,
    info: FileInfo
}

#[derive(Clone)]
struct PrintUpdateInfo {
    update_rx: UpdateRx,
    progress: ProgressType,
    info: FileInfo,
    progress_bar: ProgressBarType
}

impl Downloader {
    pub fn new(uuid: &Uuid, sender_pubkey: Rsa<Public>, info: &FileInfo) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        return Self {
            uuid: uuid.clone(),
            info: info.clone(),
            threads: None,
            chunks_completed: Arc::new(RwLock::new(0)),
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            sender_pubkey,
            update_rx: Arc::new(RwLock::new(rx)),
            update_tx: Arc::new(RwLock::new(tx)),
            update_thread: Arc::new(Mutex::new(None)),
            file_lock: Arc::new(Mutex::new(false)),
            progress_bar: Arc::new(RwLock::new(None)),
        };
    }

    pub async fn initialize(&mut self, max_chunks: u64) -> anyhow::Result<()> {
        let info = self.info.clone();
        let workers = self.workers.clone();
        let uuid = self.uuid.clone();
        let sender_pubkey = self.sender_pubkey.clone();
        let file_lock = self.file_lock.clone();
        let update_thread = self.update_thread.clone();
        let listen_info = ListenProgInfo {
            progress: self.progress.clone(),
            update_tx: self.update_tx.clone(),
            chunks_completed: self.chunks_completed.clone(),
            info: info.clone()
        };

        let update_info = PrintUpdateInfo {
            info: info.clone(),
            progress: self.progress.clone(),
            progress_bar: self.progress_bar.clone(),
            update_rx: self.update_rx.clone()
        };

        let to_spawn = std::cmp::min(CONCURRENT_THREADS, max_chunks);
        self.threads = Some(to_spawn);

        let e = tokio::spawn(async move {
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
                )?;
                state.push(worker);
            }

            drop(state);

            trace!("Listening for progress updates...");
            let state = workers.read().await;
            for el in state.iter() {
                let rec = el.progress_channel.clone();
                let index = el.get_working_id();

                Downloader::listen_for_progress_updates(listen_info.clone(), rec, index)?;
            }

            drop(state);

            trace!("Waiting for printing updates...");
            let handle = Downloader::print_updates(Some(to_spawn), update_info.clone()).await?;

            let mut state = update_thread.lock().await;
            *state = Some(handle);

            drop(state);
            Ok(())
        });

        return e.await?;
    }

    async fn print_updates(threads: Option<u64>, info: PrintUpdateInfo) -> anyhow::Result<JoinHandle<()>> {
        let PrintUpdateInfo { info, progress, progress_bar, update_rx} = info;

        let temp = update_rx.clone();
        let temp2 = progress.clone();
        let spawned = threads;
        if spawned.is_none() {
            eprintln!("Cannot print updates if no threads are spawned.");
            return Err(anyhow!("Cannot print updates with no threads spawned"));
        }

        let spawned = spawned.unwrap();
        let spawned = usize::try_from(spawned)?;
        let max_size = info.size;

        let pb = ProgressBar::new(max_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        progress_bar.write().await.replace(pb);

        let name = info.filename.clone();
        let pb_arc = progress_bar.clone();

        let e = tokio::spawn(async move {
            let state = temp.read().await;
            while let Ok(should_run) = state.recv() {
                if !should_run {
                    let state = pb_arc.read().await;
                    if state.is_some() {
                        let pb = state.as_ref().unwrap();
                        pb.finish_with_message(format!("Done downloading file {}", name));
                    }

                    drop(state);
                    break;
                }

                let prog = temp2.read().await;

                let entries = prog.len();
                let left = spawned - entries;

                let mut percentages: Vec<f32> = prog.iter().map(|e| e.1.clone()).collect();
                for _ in 0..left {
                    percentages.push(0 as f32);
                }

                let avg = get_avg(&percentages);
                if avg.is_err() {
                    warn!("Could not get avg: {}", avg.unwrap_err());
                    continue;
                }

                let avg = avg.unwrap();
                let curr = std::cmp::min(max_size, (avg * (max_size as f32)).floor() as u64);

                let state = pb_arc.read().await;
                if state.is_some() {
                    let pb = state.as_ref().unwrap();
                    pb.set_position(curr);
                }

                drop(state);
            }

            drop(state);
        });

        return Ok(e);
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

        self.update_tx.read().await.send(false)?;

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

    fn listen_for_progress_updates(
        info: ListenProgInfo,
        rec: Arc<RwLock<Receiver<f32>>>,
        index: u64,
    ) -> anyhow::Result<JoinHandle<()>> {
        let ListenProgInfo { chunks_completed, info, progress, update_tx} = info;
        let prog_arc = progress.clone();
        let tx_arc = update_tx.clone();
        let chunks_arc = chunks_completed.clone();
        let file_arc = Arc::new(RwLock::new(info.clone()));

        let e = tokio::spawn(async move {
            let rec_state = rec.read().await;

            while let Ok(prog) = rec_state.recv() {
                let mut state = prog_arc.write().await;
                let tx = tx_arc.read().await;

                state.insert(index, prog);

                let e = tx.send(true);
                if e.is_err() {
                    warn!("Could not send update: {}", e.unwrap_err());
                }

                if prog == 1.0 {
                    Downloader::on_worker_done(&chunks_arc, &file_arc).await;
                }
                drop(tx);
                drop(state);
            }

            trace!("Downloader worker {} finished.", index);
        });

        return Ok(e);
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
