use anyhow::anyhow;
use log::trace;
use packets::file::types::FileInfo;
use uuid::Uuid;

use crate::util::consts::CONCURRENT_THREADS;

use super::worker::UploadWorker;

#[derive(Debug)]
pub struct Uploader {
    file: FileInfo,
    uuid: Uuid,
    threads: Option<u64>,
    workers: Vec<UploadWorker>
}

impl Uploader {
    pub fn new(uuid: &Uuid, file: &FileInfo) -> Self {
        return Self {
            uuid: uuid.clone(),
            file: file.clone(),
            threads: None,
            workers: Vec::new()
        }
    }

    pub fn start(&mut self, threads: u64) -> anyhow::Result<()> {
        if !self.workers.is_empty() {
            return Err(anyhow!("Workers already spawned."));
        }

        self.threads = Some(threads);
        let to_spawn = std::cmp::min(CONCURRENT_THREADS, threads);

        trace!("Spawning {} workers" , to_spawn);
        for i in 0..to_spawn {
            let worker = UploadWorker::new(i, self.uuid, self.file.clone());
            self.workers.push(worker);
        }

        Ok(())
    }
}