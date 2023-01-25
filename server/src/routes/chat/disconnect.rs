use uuid::Uuid;

use crate::{utils::types::{Users, UsersList}, file::consts::{USERS_LIST, USERS}};

pub async fn user_disconnected(my_id: Uuid) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    USERS.write().await.remove(&my_id);
    let mut e = USERS_LIST.write().await;
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
