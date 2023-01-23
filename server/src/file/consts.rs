use lazy_static::lazy_static;

use super::types::*;

lazy_static! {
    pub static ref PENDING_UPLOADS: PendingUploads = PendingUploads::default();
    pub static ref UPLOADING_FILES: FileControllers = FileControllers::default();
}