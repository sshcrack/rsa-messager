use std::sync::{atomic::AtomicBool, Arc};

use clap::{arg, command, Parser};
use futures_util::{stream::{SplitSink, SplitStream}, lock::Mutex};
use openssl::{pkey::Private, rsa::Rsa};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::{RwLock, mpsc::UnboundedSender}};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use uuid::Uuid;

pub type Keypair = Arc<RwLock<Option<Rsa<Private>>>>;
pub type BaseUrl = Arc<RwLock<String>>;
pub type SendDisabled = Arc<AtomicBool>;
pub type ReceiveInput = Arc<AtomicBool>;
pub type UserId = Arc<RwLock<Option<Uuid>>>;
pub type Receiver = Arc<RwLock<Option<Uuid>>>;
pub type WebSocketGeneral = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type TXChannel = SplitSink<WebSocketGeneral, Message>;
pub type TXChannelArc = Arc<Mutex<Option<TXChannel>>>;

pub type RXChannel = SplitStream<WebSocketGeneral>;

pub type ReceiveRX = UnboundedReceiverStream<String>;
pub type ReceiveTX = UnboundedSender<String>;

#[derive(Serialize, Deserialize)]
pub struct UserInfoBasic {
    pub name: Option<String>,
    pub public_key: Option<String>,
}

/// An client designed to communicate via rsa to other clients
#[derive(Parser, Debug)]
#[command(author="sshcrack", about="An client designed to communicate via rsa to other clients", long_about = None)]
pub struct Args {
    /// The receiver of your messages
    #[arg(long, short = 'r')]
    pub receiver: Option<String>,

    /// Number of times to greet
    #[arg(short = 'a', long)]
    pub address: Option<String>,

    /// Number of times to greet
    #[arg(short = 'n', long)]
    pub name: Option<String>,

    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Message,
}
