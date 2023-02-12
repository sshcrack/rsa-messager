use std::{sync::Arc, collections::HashMap};

use clap::Parser;
use openssl::{pkey::Public, rsa::Rsa};
use packets::other::info::UserInfoBasic;
use tokio::sync::{mpsc::{self, UnboundedSender}, RwLock};
use uuid::Uuid;
use warp::ws::Message;

pub struct UserInfo {
    pub sender: mpsc::UnboundedSender<Message>,
    pub name: Option<String>,
    pub public_key: Option<Rsa<Public>>,
}

impl UserInfo {
    pub fn to_basic(&self) -> UserInfoBasic {
        UserInfoBasic { name: self.name.clone(), public_key: self.public_key.clone() }
    }
}

pub type Users = Arc<RwLock<HashMap<Uuid, UserInfo>>>;
pub type UsersList = Arc<RwLock<Vec<Uuid>>>;

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