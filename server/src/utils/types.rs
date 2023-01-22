use std::{sync::Arc, collections::HashMap};

use clap::Parser;
use futures_util::lock::Mutex;
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc::{self, UnboundedSender}, RwLock};
use uuid::Uuid;
use warp::ws::Message;


#[derive(Serialize, Deserialize)]
pub struct UserInfoBasic {
    pub name: Option<String>,
    pub public_key: Option<String>,
}

pub struct UserInfo {
    pub sender: mpsc::UnboundedSender<Message>,
    pub name: Option<String>,
    pub public_key: Option<String>,
}

pub type Users = Arc<RwLock<HashMap<Uuid, UserInfo>>>;
pub type UsersList = Arc<Mutex<Vec<Uuid>>>;

pub type TXChannel = UnboundedSender<Message>;

/// A server to host rsa-encrypted messaging between clients
#[derive(Parser, Debug)]
#[command(author="sshcrack", about="A server to host rsa-encrypted messaging between clients", long_about = None)]
pub struct Args {
    /// Address to bind to
    #[arg(long, short='b')]
    pub bind: Option<std::net::IpAddr>,

    /// Specifies on which port server should listen to
    #[arg(
        short='p',
        long,
        default_value_t = 3030
    )]
    pub port: u16,
}