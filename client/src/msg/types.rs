use std::sync::{Arc, atomic::AtomicBool};

use serde::{Serialize, Deserialize};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};


pub type SendDisabled = Arc<AtomicBool>;
pub type UserId = Arc<RwLock<Option<String>>>;
pub type Receiver = Arc<RwLock<Option<String>>>;
pub type WebSocketGeneral = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Serialize, Deserialize)]
pub struct UserInfoBasic {
    pub name: Option<String>,
    pub public_key: Option<String>
}