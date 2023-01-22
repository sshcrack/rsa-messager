use std::sync::{Arc, atomic::AtomicBool};
use futures_util::lock::Mutex;
use tokio::sync::RwLock;

use lazy_static::lazy_static;

use super::types::*;

pub const UUID_SIZE: usize = 16;
lazy_static! {
    pub static ref BASE_URL: BaseUrl = Arc::new(RwLock::new("".to_string()));
    pub static ref CURR_ID: UserId = UserId::default();
    pub static ref SEND_DISABLED: SendDisabled = Arc::new(AtomicBool::new(true));
    pub static ref RECEIVER: ReceiverArc = ReceiverArc::new(RwLock::new(None));
    pub static ref KEYPAIR: Keypair = Arc::new(RwLock::new(None));

    pub static ref TX_CHANNEL: TXChannelArc = Arc::new(Mutex::new(None));

    pub static ref RECEIVE_TX: Arc<RwLock<Option<ReceiveTX>>> = Arc::new(RwLock::new(None));
    pub static ref RECEIVE_RX: Arc<RwLock<Option<ReceiveRX>>> = Arc::new(RwLock::new(None));

    pub static ref RECEIVE_INPUT: ReceiveInput = Arc::new(AtomicBool::new(false));
}
