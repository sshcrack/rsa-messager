use uuid::Uuid;

use crate::utils::types::{Users, UsersList};

pub async fn user_disconnected(my_id: Uuid, users: &Users, users_list: &UsersList) {
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
