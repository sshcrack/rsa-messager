use warp::{reply, hyper::StatusCode};

use crate::file::consts::USERS_LIST;


pub async fn on_list() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let current = USERS_LIST.read().await;
    let vec = current.to_vec();
    let mut vec_str: Vec<String> = Vec::new();

    for el in vec {
        vec_str.push(el.to_string())
    }

    let out_str = vec_str.join(",");
    return Ok(Box::new(reply::with_status(
        out_str,
        StatusCode::OK,
    )));
}