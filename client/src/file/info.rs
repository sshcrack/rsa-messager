use uuid::Uuid;

pub struct FileRequest {
    filename: String,
    uuid: Uuid,
    receiver: Uuid,
    sender: Uuid
}

pub struct FileInfo {
    filename: String,
    uuid: Uuid,
    size: usize,
    receiver: Uuid,
    sender: Uuid
}