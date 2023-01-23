use uuid::Uuid;

pub struct FileRequest {
    pub filename: String,
    pub uuid: Uuid,
    pub receiver: Uuid,
    pub sender: Uuid
}

pub struct FileInfo {
    pub filename: String,
    pub uuid: Uuid,
    pub size: usize,
    pub receiver: Uuid,
    pub sender: Uuid
}