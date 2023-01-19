use std::sync::{Arc, RwLock};

use futures_util::lock::Mutex;
use tokio::{net::TcpStream};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};


pub type UserId = Arc<RwLock<Option<String>>>;
pub type Receiver = Arc<Mutex<String>>;
pub type WebSocketGeneral = WebSocketStream<MaybeTlsStream<TcpStream>>;