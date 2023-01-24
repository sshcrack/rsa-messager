use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileRequest {
    pub filename: String,
    pub receiver: Uuid,
    pub sender: Uuid
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub receiver: Uuid,
    pub sender: Uuid,
    pub secret: u64
}