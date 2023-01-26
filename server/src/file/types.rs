use std::{sync::Arc, collections::HashMap};
use packets::file::types::FileInfo;

use tokio::sync::RwLock;
use uuid::Uuid;

use super::controller::index::Controller;

pub type FileControllers = Arc<RwLock<HashMap<Uuid, FileInfo>>>;
pub type PendingUploads = Arc<RwLock<HashMap<Uuid, FileInfo>>>;