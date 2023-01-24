use uuid::Uuid;

use crate::web::user_info::get_user_info;

use super::msg::get_input;

pub async fn uuid_to_name(uuid: Uuid) -> anyhow::Result<String> {
    let info = get_user_info(&uuid).await?;

    if info.name.is_some() {
        return Ok(info.name.unwrap());
    }

    return Ok(uuid.to_string());
}

pub async fn wait_confirm() -> anyhow::Result<bool> {
    let accepted:bool;
    loop {
        let input = get_input().await?;

        let input = input.to_lowercase();
        if !input.eq("y") && !input.eq("n") {
            eprintln!("You have to enter either y/n");
            continue;
        }

        accepted = input.eq("y");
        break;
    }

    return Ok(accepted);
}

pub fn get_avg(vec: &Vec<f32>) -> anyhow::Result<f32> {
    let mut sum = 0 as f32;
    for e in vec {
        sum += e;
    }

    let res = sum / (vec.len() as f32);
    return Ok(res);
}