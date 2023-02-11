use std::path::PathBuf;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileRequest {
    pub filename: String,
    pub receiver: Uuid,
    pub sender: Uuid
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
    pub filename: String,
    pub size: u64,
    pub receiver: Uuid,
    pub sender: Uuid,
    pub hash: Vec<u8>
}