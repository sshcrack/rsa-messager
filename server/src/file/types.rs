use std::sync::Arc;

use tokio::sync::RwLock;

use super::info::{FileRequest, FileInfo};

pub type FileRequestList = Arc<RwLock<Vec<FileRequest>>>;
pub type FileInfoList = Arc<RwLock<Vec<FileInfo>>>;