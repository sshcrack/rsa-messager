use futures_util::{StreamExt, SinkExt, TryFutureExt};
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::WebSocket;

use crate::{utils::types::{UserInfo}, routes::chat::{disconnect::user_disconnected, messages::index::user_message}, file::consts::{USERS_LIST, USERS}};


pub async fn user_connected(ws: WebSocket) {
    // Use a counter to assign a new unique ID for this user.
    let user_id = Uuid::new_v4();
    let mut list_lock = USERS_LIST.write().await;
    list_lock.push(user_id);

    drop(list_lock);

    info!("new chat user: {}", user_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {:?}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    USERS.write().await.insert(
        user_id,
        UserInfo {
            sender: tx.clone(),
            public_key: None,
            name: None,
        },
    );

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {:?}", user_id, e);
                break;
            }
        };
        let e = user_message(user_id, msg, &tx).await;
        if e.is_err() {
            let x = e.unwrap_err();
            eprintln!("WebsocketErr: {:#?}", x);
        }
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id).await;
}
