use std::{collections::HashMap, fmt::Write, sync::Arc};

use anyhow::anyhow;
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
    file::tools::WorkerProgress,
    util::{tools::get_avg, arcs::get_concurrent_threads},
};

use super::worker::UploadWorker;
type WorkersType = Arc<RwLock<Vec<UploadWorker>>>;
type ProgressMap = HashMap<u64, f32>;
type ProgressType = Arc<RwLock<ProgressMap>>;
type UpdateThreadType = Arc<Mutex<Option<JoinHandle<()>>>>;

#[derive(Debug)]
pub struct Uploader {
    info: FileInfo,
    uuid: Uuid,

    threads: Option<u64>,

    workers: WorkersType,
    progress: ProgressType,

    receiver_key: Rsa<Public>,

    update_thread: UpdateThreadType,

    worker_tx: Arc<RwLock<UnboundedSender<WorkerProgress>>>,
    worker_rx: Arc<RwLock<UnboundedReceiverStream<WorkerProgress>>>,
}

impl Uploader {
    pub fn new(uuid: &Uuid, receiver_key: Rsa<Public>, file: &FileInfo) -> Self {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();

        let worker_rx = UnboundedReceiverStream::new(worker_rx);
        return Self {
            uuid: uuid.clone(),
            info: file.clone(),
            threads: None,
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            receiver_key,
            update_thread: Arc::new(Mutex::new(None)),
            worker_rx: Arc::new(RwLock::new(worker_rx)),
            worker_tx: Arc::new(RwLock::new(worker_tx)),
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

        let handle = self.listen_for_progress_update()?;

        let mut state = self.update_thread.lock().await;
        *state = Some(handle);

        drop(state);
        let to_spawn = get_concurrent_threads().await.min(max_threads);

        self.threads = Some(to_spawn);
        trace!("Spawning {} workers", to_spawn);
        let mut state = self.workers.write().await;
        let mut state_prog = self.progress.write().await;

        for i in 0..to_spawn {
            trace!("Spawning upload worker with Thread_Indx {}", i);
            let mut worker = UploadWorker::new(
                i,
                self.uuid,
                self.receiver_key.clone(),
                self.info.clone(),
                self.worker_tx.clone(),
            )?;

            let e = worker.start(i).await;
            if e.is_err() {
                drop(state);
                drop(state_prog);
                return Err(e.unwrap_err());
            }

            state.push(worker);
            state_prog.insert(i, 0 as f32);
        }
        trace!("Dropping state workers...");
        drop(state);
        drop(state_prog);
        Ok(())
    }

    fn print_update(pb: &ProgressBar, progress: &ProgressMap, max_size: u64) {
        let max_chunks = get_max_chunks(max_size);
        let mut percentages: Vec<f32> = progress.values().map(|e| e.clone()).collect();

        let prog_len: u64 = progress.len().try_into().unwrap();
        let mut left = 0 as u64;
        if max_chunks >= prog_len {
            left = max_chunks - prog_len
        }

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

    fn listen_for_progress_update(&self) -> anyhow::Result<JoinHandle<()>> {
        let temp = self.progress.clone();
        let worker_rx_arc = self.worker_rx.clone();

        let max_size = self.info.size;

        let e = tokio::spawn(async move {
            let pb = ProgressBar::new(max_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

            let mut worker_rx = worker_rx_arc.write().await;
            while let Some(el) = worker_rx.next().await {
                let WorkerProgress { chunk, progress } = el;
                let progress = progress.min(1 as f32);

                let mut state = temp.write().await;
                let old_prog = state.get(&chunk).unwrap_or(&(0 as f32)).to_owned();
                state.insert(chunk, progress);

                if progress >= 1.0 && old_prog < 1.0 {
                    trace!("Upload worker {} finished.", chunk);
                }

                Uploader::print_update(&pb, &state, max_size);
                drop(state);
            }
            drop(worker_rx);
        });

        return Ok(e);
    }

    pub async fn on_next(&self) -> anyhow::Result<()> {
        let mut chunks_left = None;
        let state = self.progress.read().await;
        for i in 0..self.get_max_chunks() {
            if state.get(&i).is_none() {
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
        let mut s = self.progress.write().await;
        s.insert(chunk, 0 as f32);

        drop(s);

        let res = worker.start(chunk).await;
        drop(state);

        res?;
        Ok(())
    }

    pub fn get_max_chunks(&self) -> u64 {
        return get_max_chunks(self.info.size) as u64;
    }

    pub fn get_file_info(&self) -> FileInfo {
        return self.info.clone();
    }

    pub async fn is_done(&self) -> bool {
        let max_chunks = self.get_max_chunks();
        let completed = self.get_chunks_completed().await;

        return max_chunks as usize == completed;
    }

    pub async fn get_chunks_completed(&self) -> usize {
        let state = self.progress.read().await;
        let done: Vec<&f32> = state
            .values()
            .filter(|e| e.to_owned().to_owned() >= 1 as f32)
            .collect();
        let done = done.len();

        drop(state);
        return done;
    }

    #[allow(dead_code)]
    pub async fn get_chunks_processing(&self) -> Vec<u64> {
        let state = self.progress.read().await;
        let processing: Vec<u64> = state
            .iter()
            .filter(|e| e.1.to_owned().to_owned() < 1 as f32)
            .map(|e| e.0.to_owned())
            .collect();

        drop(state);
        return processing;
    }

    pub async fn get_chunks_spawned(&self) -> Vec<u64> {
        let state = self.progress.read().await;
        let processing: Vec<u64> = state
            .iter()
            .map(|e| e.0.to_owned())
            .collect();

        drop(state);
        return processing;
    }
}
