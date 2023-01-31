use std::{collections::HashMap, fmt::Write, ops::Add, sync::Arc};

use anyhow::anyhow;
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

use super::worker::UploadWorker;

#[derive(Debug)]
pub struct Uploader {
    info: FileInfo,
    uuid: Uuid,

    chunks_completed: Arc<RwLock<u64>>,
    chunks_processing: Arc<RwLock<Vec<u64>>>,
    threads: Option<u64>,

    workers: Arc<RwLock<Vec<UploadWorker>>>,
    progress: Arc<RwLock<HashMap<u64, f32>>>,

    receiver_key: Rsa<Public>,

    update_tx: Arc<RwLock<Sender<bool>>>,
    update_rx: Arc<RwLock<Receiver<bool>>>,
    update_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    progress_bar: Arc<RwLock<Option<ProgressBar>>>,
}

impl Uploader {
    pub fn new(uuid: &Uuid, receiver_key: Rsa<Public>, file: &FileInfo) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        return Self {
            uuid: uuid.clone(),
            info: file.clone(),
            threads: None,
            chunks_completed: Arc::new(RwLock::new(0)),
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            receiver_key,
            update_rx: Arc::new(RwLock::new(rx)),
            update_tx: Arc::new(RwLock::new(tx)),
            update_thread: Arc::new(Mutex::new(None)),
            progress_bar: Arc::new(RwLock::new(None)),
            chunks_processing: Arc::new(RwLock::new(Vec::new()))
        };
    }

    pub async fn start(&mut self, max_threads: u64) -> anyhow::Result<()> {
        if self.info.path.is_none() {
            return Err(anyhow!(
                "Can not start uploader when download path is none."
            ));
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
        let mut state_processing = self.chunks_processing.write().await;

        for i in 0..to_spawn {
            trace!("Spawning upload worker with Thread_Indx {}", i);
            let mut worker = UploadWorker::new(i, self.uuid, self.receiver_key.clone(), self.info.clone())?;

            let e = worker.start(i).await;
            if e.is_err() {
                drop(state);
                drop(state_processing);
                return Err(e.unwrap_err());
            }

            state.push(worker);
            state_processing.push(i);
        }
        trace!("Dropping state workers...");
        drop(state);
        drop(state_processing);

        let state = self.workers.read().await;
        for el in state.iter() {
            let rec = el.progress_channel.clone();
            let index = el.get_working_id();

            let temp = self.progress.clone();
            let temp1 = self.update_tx.clone();
            let temp2 = self.chunks_completed.clone();

            tokio::spawn(async move {
                let rec_state = rec.read().await;

                while let Ok(prog) = rec_state.recv() {
                    let mut state = temp.write().await;
                    let tx = temp1.read().await;

                    state.insert(index, prog);
                    let e = tx.send(true);
                    if e.is_err() {
                        warn!("Could not send update: {}", e.unwrap_err());
                    }

                    if prog == 1.0 {
                        let mut s = temp2.write().await;
                        *s = s.add(1 as u64);

                        drop(s);
                    }
                    drop(tx);
                    drop(state);
                }
                trace!("Upload worker {} finished.", index);
            });
        }

        drop(state);

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

        let max_size = self.info.size;
        let pb = ProgressBar::new(max_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        self.progress_bar.write().await.replace(pb);

        let pb_arc = self.progress_bar.clone();
        let e = tokio::spawn(async move {
            let state = temp.read().await;
            while let Ok(should_run) = state.recv() {
                if !should_run {
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

                drop(prog);
            }

            drop(state);
        });

        return Ok(e);
    }

    pub async fn on_next(&self) -> anyhow::Result<()> {
        let mut chunks_left = None;
        trace!("Processing read");
        let state = self.chunks_processing.read().await;
        for i in 0..self.get_max_chunks() {
            if !state.contains(&i) {
                chunks_left = Some(i);
                break;
            }
        }

        drop(state);
        if chunks_left.is_none() {
            return Err(anyhow!("No chunks left."));
        }

        return self.start_upload(chunks_left.unwrap()).await;
    }

    pub async fn start_upload(&self, chunk: u64) -> anyhow::Result<()> {
        trace!("Starting to upload chunk {}", chunk);

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
            eprint!("Unknown Error occurred while getting worker for new upload.");
            return Err(anyhow!("Unknown Error when start upload"));
        }

        let worker = worker.unwrap();
        let mut s = self.chunks_processing.write().await;
        s.push(chunk);

        drop(s);

        let res = worker.start(chunk).await;
        drop(state);

        res?;
        Ok(())
    }

    pub fn get_max_chunks(&self) -> u64 {
        return get_max_chunks(self.info.size);
    }

    pub fn get_file_info(&self) -> FileInfo {
        return self.info.clone();
    }

    pub async fn get_chunks_completed(&self) -> u64 {
        let state = self.chunks_completed.read().await;
        let completed = state.clone();

        drop(state);
        return completed;
    }

    pub async fn get_chunks_processing(&self) -> Vec<u64> {
        let state = self.chunks_processing.read().await;
        let processing = state.clone();

        drop(state);
        return processing;
    }
}
