use std::{sync::Arc, collections::HashMap, ops::Add};

use anyhow::anyhow;
use crossbeam_channel::{ Receiver, Sender};
use futures_util::future::select_all;
use log::{trace, warn};
use openssl::{pkey::Public, rsa::Rsa};
use packets::file::types::FileInfo;
use tokio::{sync::{RwLock, Mutex}, task::JoinHandle};
use uuid::Uuid;

use crate::util::{consts::CONCURRENT_THREADS, tools::get_avg};

use super::worker::UploadWorker;

#[derive(Debug)]
pub struct Uploader {
    file: FileInfo,
    uuid: Uuid,

    chunks_completed: Arc<RwLock<u64>>,
    threads: Option<u64>,

    workers: Arc<RwLock<Vec<UploadWorker>>>,
    progress: Arc<RwLock<HashMap<u64, f32>>>,

    key: Rsa<Public>,

    update_tx: Arc<RwLock<Sender<bool>>>,
    update_rx: Arc<RwLock<Receiver<bool>>>,
    update_thread: Arc<Mutex<Option<JoinHandle<()>>>>
}

impl Uploader {
    pub fn new(uuid: &Uuid, key: Rsa<Public>, file: &FileInfo) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        return Self {
            uuid: uuid.clone(),
            file: file.clone(),
            threads: None,
            chunks_completed: Arc::new(RwLock::new(0)),
            workers: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            key,
            update_rx: Arc::new(RwLock::new(rx)),
            update_tx: Arc::new(RwLock::new(tx)),
            update_thread: Arc::new(Mutex::new(None))
        }
    }

    pub async fn start(&mut self, max_threads: u64) -> anyhow::Result<()> {
        let state = self.workers.read().await;

        if !state.is_empty() {
            drop(state);
            return Err(anyhow!("Workers already spawned."));
        }
        drop(state);

        let to_spawn = std::cmp::min(CONCURRENT_THREADS, max_threads);

        self.threads = Some(to_spawn);
        trace!("Spawning {} workers" , to_spawn);
        let mut state = self.workers.write().await;
        for i in 0..to_spawn {
            let mut worker = UploadWorker::new(i, self.uuid, self.key.clone(), self.file.clone())?;

            let e = worker.start(i);
            if e.is_err() {
                drop(state);
                return Err(e.unwrap_err());
            }

            state.push(worker);
        }
        println!("Dropping state workers...");
        drop(state);

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
                    let e =tx.send(true);
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
                };
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

        let e = tokio::spawn(async move {
            let state = temp.read().await;
            while let Ok(should_run) = state.recv() {
                println!("New update {}", should_run);
                if !should_run  { break; }

                let prog = temp2.read().await;

                let entries = prog.len();
                let left = spawned - entries;

                let mut percentages: Vec<f32> = prog.iter().map(|e| e.1.clone()).collect();
                for _ in 0..left {
                    percentages.push(0 as f32);
                };

                let avg = get_avg(&percentages);
                if avg.is_err() {
                    drop(prog);
                    break;
                }

                drop(prog);

                let avg = avg.unwrap() * 100 as f32;
                println!("Percentage: {}", avg)
            }

            drop(state);
        });

        return Ok(e);
    }

    pub async fn start_upload(&self, chunk: u64) -> anyhow::Result<()> {
        let mut state = self.workers.write().await;
        let futures = state.iter_mut()
            .map(|e| Box::pin(async {
                let res = e.wait_for_end().await;
                if res.is_err() {
                    eprintln!("Error when waiting for end: {}", res.unwrap_err());
                    return None;
                }
                return Some(e.get_working_id());
            }));

        let selected = select_all(futures);
        let (available_worker_id,..) = selected.await;
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
        let res = worker.start(chunk);
        drop(state);


        res?;
        Ok(())
    }

/* just for debugging purposes
    pub async fn wait_for_workers(&mut self) -> anyhow::Result<()> {
        println!("Waiting for state of workers...");
        let mut state = self.workers.write().await;
        let total = state.len();

        println!("Waiting for {} workers...", total);
        let futures = state
            .iter_mut()
            .map(|worker| worker.wait_for_end());

        trace!("Joining {} futures...", total);
        let (workers, update) = join(try_join_all(futures), self.print_updates()).await;
        drop(state);

        let update = update?;
        update.abort();

        let state = self.update_tx.read().await;
        state.send(false)?;
        drop(state);

        workers?;
        return Ok(());
    } */

}