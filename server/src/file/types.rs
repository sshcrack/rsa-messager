use std::{sync::Arc, collections::HashMap};
use packets::file::types::FileInfo;

use tokio::sync::RwLock;
use uuid::Uuid;

pub type FileControllers = Arc<RwLock<HashMap<Uuid, FileInfo>>>;
pub type PendingUploads = Arc<RwLock<HashMap<Uuid, FileInfo>>>;