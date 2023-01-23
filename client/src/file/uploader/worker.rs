use log::trace;
use packets::file::types::FileInfo;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug)]
pub struct UploadWorker {
    thread_index: u64,
    uuid: Uuid,
    file: FileInfo,
    thread: JoinHandle<()>
}

impl UploadWorker {
    pub fn new(i: u64, uuid: Uuid, file: FileInfo) -> Self {
        trace!("Spawned new worker with i: {} uuid: {}", i, uuid);

        let thread = tokio::spawn(async move {
            //TODO handle splitting file in chunks and actually send it to the server first encrypting with rsa
        });

        return UploadWorker {
            thread_index: i,
            uuid,
            file,
            thread
        };
    }
}
