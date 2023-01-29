use std::{collections::HashMap, ops::Add, sync::Arc, fmt::Write};

use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use futures_util::future::select_all;
use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use log::{trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::file::types::FileInfo;
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::util::{consts::CONCURRENT_THREADS, tools::get_avg};

use super::worker::DownloadWorker;

#[derive(Debug)]
pub struct Downloader {
    info: FileInfo,
    uuid: Uuid,

    chunks_completed: Arc<RwLock<u64>>,
    threads: Option<u64>,

    workers: Arc<RwLock<Vec<DownloadWorker>>>,
    progress: Arc<RwLock<HashMap<u64, f32>>>,

    sender_pubkey: Rsa<Public>,

    update_tx: Arc<RwLock<Sender<bool>>>,
    update_rx: Arc<RwLock<Receiver<bool>>>,
    update_thread: Arc<Mutex<Option<JoinHandle<()>>>>,

    // Bool is just a placeholder
    file_lock: Arc<Mutex<bool>>,
    progress_bar: Arc<RwLock<Option<ProgressBar>>>
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
            progress_bar: Arc::new(RwLock::new(None))
        };
    }

    pub async fn initialize(&mut self, max_threads: u64) -> anyhow::Result<()> {
        if self.info.path.is_none() {
            return Err(anyhow!("Can not initialize downloader when download path is none."));
        }

        trace!("Waiting for read...");
        let state = self.workers.read().await;

        if !state.is_empty() {
            drop(state);
            return Err(anyhow!("Workers already spawned."));
        }
        drop(state);

        let to_spawn = std::cmp::min(CONCURRENT_THREADS, max_threads);

        self.threads = Some(to_spawn);
        trace!("Spawning {} workers", to_spawn);
        let mut state = self.workers.write().await;
        for i in 0..to_spawn {
            let worker = DownloadWorker::new(
                i,
                self.uuid,
                self.sender_pubkey.clone(),
                self.info.clone(),
                self.file_lock.clone(),
            )?;
            state.push(worker);
        }

        drop(state);

        trace!("Listening for progress updates...");
        let state = self.workers.read().await;
        for el in state.iter() {
            let rec = el.progress_channel.clone();
            let index = el.get_working_id();

            self.listen_for_progress_updates(rec, index)?;
        }

        drop(state);

        trace!("Waiting for printing updates...");
        let handle = self.print_updates().await?;

        let mut state = self.update_thread.lock().await;
        *state = Some(handle);

        drop(state);
        Ok(())
    }

    async fn print_updates(&self) -> anyhow::Result<JoinHandle<()>> {
        let temp = self.update_rx.clone();
        let temp2 = self.progress.clone();
        let spawned = self.threads;
        if spawned.is_none() {
            eprintln!("Cannot print updates if no threads are spawned.");
            return Err(anyhow!("Cannot print updates with no threads spawned"));
        }

        let spawned = spawned.unwrap();
        let spawned = usize::try_from(spawned)?;
        let max_size= self.info.size;

        let pb = ProgressBar::new(max_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        self.progress_bar.write().await.replace(pb);

        let name = self.info.filename.clone();
        let pb_arc = self.progress_bar.clone();

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
        &self,
        rec: Arc<RwLock<Receiver<f32>>>,
        index: u64,
    ) -> anyhow::Result<JoinHandle<()>> {
        let prog_arc = self.progress.clone();
        let tx_arc = self.update_tx.clone();
        let chunks_arc = self.chunks_completed.clone();

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
                    Downloader::on_worker_done(&chunks_arc).await;
                }
                drop(tx);
                drop(state);
            }

            trace!("Downloader worker {} finished.", index);
        });

        return Ok(e)
    }

    async fn on_worker_done(chunks_completed: &Arc<RwLock<u64>>) {
        let mut s = chunks_completed.write().await;
        *s = s.add(1 as u64);

        drop(s);
    }
}
