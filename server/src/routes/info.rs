use std::{collections::HashMap, str::FromStr};

use uuid::Uuid;
use warp::{
    hyper::{Response, StatusCode},
    reply,
};

use crate::file::consts::USERS;

pub async fn on_info(p: HashMap<String, String>) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
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
    let basic = info.to_basic().serialize();
    drop(state);
    if basic.is_err() {
        return Ok(Box::new(
            Response::builder().body(String::from("Could not get user info"))
        ))
    }

    let basic = basic.unwrap();
    return Ok(Box::new(reply::with_status(basic, StatusCode::OK)));
}
