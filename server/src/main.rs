
use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use file::consts::CHUNK_DIR;
use routes::router::serve_routes;
use tokio::fs::remove_dir_all;
use crate::utils::types::*;

mod utils;
mod routes;
mod file;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let p = CHUNK_DIR.as_path();
    if p.is_dir() {
        let e = remove_dir_all(p).await;
        if e.is_err() {
            eprintln!("Could not remove chunk dir: {}", e.unwrap_err());
        }
    }

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let args = Args::parse();

    let addr = args.bind.unwrap_or(IpAddr::V4(Ipv4Addr::new(127,0,0,1)));
    let port = args.port;

    serve_routes((addr, port)).await;
}
