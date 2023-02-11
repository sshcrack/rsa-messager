use std::{sync::{atomic::AtomicBool, Arc}, collections::HashMap};

use async_channel::{Receiver, Sender};
use clap::{arg, command, Parser};
use futures_util::{stream::{SplitSink, SplitStream}, lock::Mutex};
use openssl::{pkey::Private, rsa::Rsa};
use packets::file::types::FileInfo;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use uuid::Uuid;

use crate::file::{uploader::index::Uploader, downloader::index::Downloader};

pub type Keypair = Arc<RwLock<Option<Rsa<Private>>>>;
pub type ConcurrentThreads= Arc<RwLock<u64>>;
pub type BaseUrl = Arc<RwLock<String>>;
pub type UseTls = Arc<RwLock<bool>>;
pub type SendDisabled = Arc<AtomicBool>;
pub type ReceiveInput = Arc<AtomicBool>;
pub type UserId = Arc<RwLock<Option<Uuid>>>;
pub type ReceiverArc = Arc<RwLock<Option<Uuid>>>;
pub type WebSocketGeneral = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type TXChannel = SplitSink<WebSocketGeneral, Message>;
pub type TXChannelArc = Arc<Mutex<Option<TXChannel>>>;

pub type RXChannel = SplitStream<WebSocketGeneral>;

pub type ReceiveRX = Receiver<String>;
pub type ReceiveTX = Sender<String>;

pub type FileUploads = Arc<RwLock<HashMap<Uuid, Uploader>>>;
pub type FileDownloads = Arc<RwLock<HashMap<Uuid, Downloader>>>;
pub type PendingFiles = Arc<RwLock<HashMap<Uuid, FileInfo>>>;

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

    /// The address this client should connect to
    #[arg(short = 'a', long)]
    pub address: Option<String>,

    /// Weither the client should use http:// and wss:// (tls)
    #[arg(short = 's', long)]
    pub secure: Option<bool>,

    /// Name of the client
    #[arg(short = 'n', long)]
    pub name: Option<String>,

    /// Threads to use when downloading
    #[arg(short = 't', long)]
    pub threads: Option<usize>,

    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Message,
}
