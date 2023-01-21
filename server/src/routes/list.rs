use crate::utils::types::UsersList;


pub async fn on_list(users_list: UsersList) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let current = users_list.lock().await;
    let vec = current.to_vec();
    let mut vec_str: Vec<String> = Vec::new();

    for el in vec {
        vec_str.push(el.to_string())
    }
    return Ok(Box::new(warp::reply::json(&vec_str)));
}