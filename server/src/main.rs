// #![deny(warnings)]
use std::collections::HashMap;
use std::sync:: Arc;

use futures_util::lock::Mutex;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type UsersList = Arc<Mutex<Vec<Uuid>>>;

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.

    let users_list_org = UsersList::default();
    let users_list_org_clone = users_list_org.clone();


    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    let users_list = warp::any().map(move || users_list_org.clone());
    let users_list_second = warp::any().map(move || users_list_org_clone.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .and(users_list)
        .map(|ws: warp::ws::Ws, users, users_list| {
            // This will call our function if the handshake succeeds.

            ws.on_upgrade(move |socket| {
                return user_connected(socket, users, users_list);
            })
        });


    // GET / -> index html
    let index = warp::path::end()
    .map(|| warp::reply::html(INDEX_HTML));

    let list_route = warp::path("list")
        .and(warp::path::end())
        .and(users_list_second)
        .and_then(on_list);

    let routes = warp::get().and(
        index
        .or(chat)
        .or(list_route)
    );


    println!("Listening on http://localhost:3030 !");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn on_list(users_list: UsersList) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let current = users_list.lock().await;
    let vec = current.to_vec();
    let mut vec_str: Vec<String> = Vec::new();

    for el in vec { vec_str.push(el.to_string()) }
    return Ok(Box::new(warp::reply::json(&vec_str)));
}

async fn user_connected(ws: WebSocket, users: Users, users_list: UsersList) {
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
    let res = tx.send(Message::text(format!("UID:{}", user_id)));
    if res.is_err() {
        eprintln!("could not send uid {:#?}", res.unwrap_err())
    }

    users.write().await.insert(user_id, tx);

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
        user_message(user_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id, &users).await;
}

async fn user_message(my_id: Uuid, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: Uuid, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);

        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }

        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };

        ws.onmessage = function(msg) {
            message(msg.data);
        };

        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';

            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
