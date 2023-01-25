use crate::util::arcs::use_tls;

pub async fn get_web_protocol() -> String {
    let tls = use_tls().await;

    if tls {
        return "https:".to_owned();
    }

    return "http:".to_owned()
}

pub async fn get_ws_protocol() -> String {
    let tls = use_tls().await;

    if tls {
        return "wss:".to_owned();
    }

    return "ws:".to_owned()
}