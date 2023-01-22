use crate::{util::consts::RECEIVER, input::receiver::select_receiver};

pub async fn on_receiver(_line: &str) -> anyhow::Result<()> {
    let new_rec = select_receiver().await;
    if new_rec.is_err() {
        return Err(new_rec.unwrap_err());
    }

    let new_rec = new_rec.unwrap();

    let mut state = RECEIVER.write().await;
    *state = Some(new_rec.clone());

    drop(state);
    return Ok(());
}
