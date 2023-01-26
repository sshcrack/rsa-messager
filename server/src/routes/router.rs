use std::{collections::HashMap, net::SocketAddr};

use crate::routes::{chat::connect::user_connected, files::upload::on_upload, index::get_index};
use colorize::AnsiColor;
use packets::consts::CHUNK_SIZE;
use warp::Filter;

use super::{info::on_info, list::on_list};

pub async fn serve_routes(addr: impl Into<SocketAddr>) {
    // GET / -> index html
    let index = warp::path::end().and_then(get_index);

    let list_route = warp::path("list").and(warp::path::end()).and_then(on_list);

    let info_route = warp::path("info")
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(on_info);
    // GET /chat -> websocket upgrade

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // This will call our function if the handshake succeeds.

            ws.on_upgrade(move |socket| {
                return user_connected(socket);
            })
        });

    let upload_route = warp::path!("file" / "upload")
        .and(warp::body::content_length_limit(CHUNK_SIZE))
        .and(warp::body::stream())
        .and_then(on_upload);

    let routes = warp::get()
        .and(index.or(chat).or(list_route).or(info_route))
        .or(
            warp::post().and(upload_route)
        );
    let addr: SocketAddr = addr.into();

    let url = format!("http://{}", addr).blue();
    println!("{} {} !", "Listening on".b_black(), url);
    warp::serve(routes).run(addr).await;
}
