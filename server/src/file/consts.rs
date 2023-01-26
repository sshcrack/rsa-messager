use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

use crate::utils::types::{Users, UsersList};

use super::types::*;

lazy_static! {
    pub static ref PENDING_UPLOADS: PendingUploads = PendingUploads::default();
    pub static ref UPLOADING_FILES: FileControllers = FileControllers::default();
    pub static ref USERS: Users = Users::default();
    pub static ref USERS_LIST: UsersList = UsersList::default();
    pub static ref CHUNK_DIR: PathBuf = Path::new("chunks").to_path_buf();
}