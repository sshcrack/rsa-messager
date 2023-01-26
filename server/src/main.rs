
use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use routes::router::serve_routes;
use crate::utils::types::*;

mod utils;
mod routes;
mod file;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "server,tungestenit,packets");
    }

    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let args = Args::parse();

    let addr = args.bind.unwrap_or(IpAddr::V4(Ipv4Addr::new(127,0,0,1)));
    let port = args.port;

    serve_routes((addr, port)).await;
}
