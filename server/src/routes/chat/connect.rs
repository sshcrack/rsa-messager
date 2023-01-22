use futures_util::{StreamExt, SinkExt, TryFutureExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::WebSocket;

use crate::{utils::types::{Users, UsersList, UserInfo}, routes::chat::{disconnect::user_disconnected, messages::index::user_message}};


pub async fn user_connected(ws: WebSocket, users: Users, users_list: UsersList) {
    // Use a counter to assign a new unique ID for this user.
    let user_id = Uuid::new_v4();
    let mut list_lock = users_list.lock().await;
    list_lock.push(user_id);

    drop(list_lock);

    println!("new chat user: {}", user_id);

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
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(
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
                eprintln!("websocket error(uid={}): {}", user_id, e);
                break;
            }
        };
        let e = user_message(user_id, msg, &users, &tx).await;
        if e.is_err() {
            eprintln!("WebsocketErr: {}", e.unwrap_err());
        }
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id, &users, &users_list).await;
}
