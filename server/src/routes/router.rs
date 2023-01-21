use std::{collections::HashMap, net::SocketAddr};

use colorize::AnsiColor;
use warp::Filter;
use crate::{utils::types::{Users, UsersList}, routes::{chat::connect::user_connected, index::get_index}};

use super::{info::on_info, list::on_list};

pub async fn serve_routes(addr: impl Into<SocketAddr>, users: Users, list: UsersList) {
    // GET / -> index html
    let index = warp::path::end()
    .and_then(get_index);

    let t_users = users.clone();
    let t_list = list.clone();

    let tf_users = warp::any().map(move || t_users.clone());
    let tf_list = warp::any().map(move || t_list.clone());

    let list_route = warp::path("list")
        .and(warp::path::end())
        .and(tf_list)
        .and_then(on_list);


    let info_route = warp::path("info")
        .and(warp::path::end())
        .and(tf_users)
        .and(warp::query::<HashMap<String, String>>())
        .and_then(on_info);
    let t_users = users.clone();
    let t_list = list.clone();

    let tf_users = warp::any().map(move || t_users.clone());
    let tf_list = warp::any().map(move || t_list.clone());

    // GET /chat -> websocket upgrade

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(tf_users)
        .and(tf_list)
        .map(|ws: warp::ws::Ws, users, users_list| {
            // This will call our function if the handshake succeeds.

            ws.on_upgrade(move |socket| {
                return user_connected(socket, users, users_list);
            })
        });


    let routes = warp::get()
    .and(
        index
        .or(chat)
        .or(list_route)
        .or(info_route)
    );


    let addr: SocketAddr = addr.into();

    let url = format!("http://{}", addr).blue();
    println!("{} {} !","Listening on".b_black(), url);
    warp::serve(routes).run(addr).await;
}
