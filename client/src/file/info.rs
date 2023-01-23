use uuid::Uuid;

struct FileRequest {
    filename: String,
    uuid: Uuid,
    receiver: Uuid,
    sender: Uuid
}

struct FileInfo {
    filename: String,
    uuid: Uuid,
    size: usize,
    receiver: Uuid,
    sender: Uuid
}