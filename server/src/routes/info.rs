use std::{collections::HashMap, str::FromStr};

use uuid::Uuid;
use warp::hyper::Response;

use crate::{utils::types::{UserInfoBasic}, file::consts::USERS};


pub async fn on_info(p: HashMap<String, String>,) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let k = p.get("id");
    if k.is_none() {
        return Ok(Box::new(
            Response::builder().body(String::from("No \"key\" param in query.")),
        ));
    }

    let k = k.unwrap();
    let uuid = Uuid::from_str(k);
    if uuid.is_err() {
        return Ok(Box::new(
            Response::builder().body(String::from("Invalid uuid")),
        ));
    }

    let uuid = uuid.unwrap();

    let state = USERS.read().await;
    let info = state.get(&uuid).clone();

    if info.is_none() {
        drop(state);
        return Ok(Box::new(
            Response::builder().body(String::from("User info is null")),
        ));
    }

    let info = info.unwrap();
    let name = &info.name;
    let pubkey = &info.public_key;

    let basic = UserInfoBasic {
        name: name.to_owned(),
        public_key: pubkey.to_owned(),
    };

    drop(state);
    return Ok(Box::new(warp::reply::json(&basic)));
}