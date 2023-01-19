// #![deny(warnings)]
use std::collections::HashMap;
use std::str::FromStr;
use std::sync:: Arc;

use futures_util::lock::Mutex;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::hyper::Response;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type UsersList = Arc<Mutex<Vec<Uuid>>>;
type PubKeyList = Arc<Mutex<HashMap<Uuid, String>>>;

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
    let users = Users::default();
    let pubkey_list_org = PubKeyList::default();


    let users_list_org_clone = users_list_org.clone();
    let pubkey_list_org_clone = pubkey_list_org.clone();


    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    let users_list = warp::any().map(move || users_list_org.clone());
    let users_list_second = warp::any().map(move || users_list_org_clone.clone());


    let pubkey_list = warp::any().map(move || pubkey_list_org.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .and(users_list)
        .and(pubkey_list)
        .map(|ws: warp::ws::Ws, users, users_list, pubkey_list| {
            // This will call our function if the handshake succeeds.

            ws.on_upgrade(move |socket| {
                return user_connected(socket, users, users_list, pubkey_list);
            })
        });


    // GET / -> index html
    let index = warp::path::end()
    .map(|| warp::reply::html(""));

    let list_route = warp::path("list")
        .and(warp::path::end())
        .and(users_list_second)
        .and_then(on_list);


    let pubkey_list = warp::any().map(move || pubkey_list_org_clone.clone());
    let pubkey_route = warp::path("pubkey")
    .and(warp::path::end())
    .and(pubkey_list)
    .and(warp::query::<HashMap<String, String>>())
    .and_then(on_pubkey);

    let routes = warp::get().and(
        index
        .or(chat)
        .or(list_route)
        .or(pubkey_route)
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

async fn on_pubkey(pubkey_list: PubKeyList, p: HashMap<String, String>) -> Result<Box<dyn warp::Reply>, warp::Rejection>  {
    let k = p.get("id");
    if k.is_none() {
        return Ok(Box::new(Response::builder().body(String::from("No \"key\" param in query."))));
    }

    let k = k.unwrap();
    let uuid = Uuid::from_str(k);
    if uuid.is_err() {
        return Ok(Box::new(Response::builder().body(String::from("Invalid uuid"))));
    }

    let uuid = uuid.unwrap();

    let state = pubkey_list.lock().await;
    let pubkey = state.get(&uuid);

    if pubkey.is_none() {
        return Ok(Box::new(Response::builder().body(String::from("Pubkey could not be found"))));
    }

    let pubkey = pubkey.unwrap().to_string();
    drop(state);

    return Ok(Box::new(Response::builder().body(pubkey)));


}

async fn user_connected(ws: WebSocket, users: Users, users_list: UsersList, pubkey_list: PubKeyList) {
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
        user_message(user_id, msg, &users, &pubkey_list).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(user_id, &users, &users_list).await;
}

async fn user_message(my_id: Uuid, msg: Message, users: &Users, pubkey_list: &PubKeyList) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    if msg.starts_with("to:") {
        handle_msg_to(my_id, msg, users).await;
        return;
    }

    if msg.starts_with("setpubkey:") {
        // TODO if i want to, check size of key here
        let msg = msg.replace("setpubkey:", "");
        let mut state = pubkey_list.lock().await;

        state.insert(my_id, msg);
        drop(state);
        println!("Pubkey set.");
    }
}


async fn handle_msg_to(my_id: Uuid, msg: &str, users: &Users) {
    let mut parts: Vec<&str> = msg.split(":").collect();
    let parts_c = parts.clone();
    let send_to = parts_c.get(1);
    if send_to.is_none() {
        println!("Invalid format");
        return;
    }

    let send_to = send_to.unwrap();
    parts.remove(0);
    parts.remove(0);

    let left = parts.join(":");

    let new_msg = format!("from:{}:{}", my_id, left);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if send_to.to_string().eq(&uid.to_string()) {
            println!("Sending {} to {}", new_msg, uid);
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: Uuid, users: &Users, users_list: &UsersList) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
    let mut e = users_list.lock().await;
    let mut i = 0;
    for el in e.clone().iter() {
        if el.eq(&my_id) {
            e.remove(i);
            break;
        }
        i += 1;
    }

    drop(e);
}