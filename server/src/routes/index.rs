use warp::reply;

pub async fn get_index() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let html = include_str!("../frontend/index.html");
    return Ok(Box::new(reply::html(html)));
}
