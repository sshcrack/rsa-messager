use anyhow::anyhow;
use crossbeam_channel::Sender;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use surf::Response;
use std::{cmp::min, sync::Arc};
use tokio::sync::RwLock;

pub async fn download_file(url: String, sender: &Sender<f32>) -> anyhow::Result<Vec<u8>> {
    let arc = Arc::new(RwLock::new(sender));

    // Reqwest setup
    let res = surf::get(url.clone())
        .await
        .or(Err(anyhow!(format!("Failed to GET from '{}'", &url))))?;
    let total_size = res.header("Content-Length");
    if total_size.is_none() {
        return Err(anyhow!(format!(
            "Failed to get content length from '{}'",
            &url
        )));
    }

    let total_size = total_size.unwrap();
    let total_size = total_size.get(0);
    if total_size.is_none() {
        return Err(anyhow!(format!(
            "Failed to get content length from '{}'",
            &url
        )));
    }

    let total_size = total_size.unwrap();
    println!("Parsing size '{}' to u64", total_size);
    let total_size = total_size.to_string().parse::<u64>()?;

    let mut buffer = Vec::new();
    let stream = BufReader::new(res);
    let arc_stream = Arc::new(RwLock::new(stream));

    let mut downloaded: u64 = 0;

    let state = arc.read().await;
    loop {
        let mut buf_state = arc_stream.write().await;
        let mut chunk = buf_state.fill_buf().await?.to_vec();
        if chunk.len() <= 0 {
            break;
        }

        drop(buf_state);
        let mut buf_state = arc_stream.write().await;

        buf_state.consume_unpin(chunk.len());
        drop(buf_state);

        buffer.append(&mut chunk);

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        state.send((new as f32) / (total_size as f32))?;
    }
    //TODO maybe useless drop?
    drop(state);
    return Ok(buffer);
}

pub async fn upload_file(
    url: String,
    buf: Vec<u8>,
    //sender: Sender<f32>
) -> anyhow::Result<Response> {
    //TODO
    /*
    let arc = Arc::new(RwLock::new(sender));

    let total_size = buf.len();
    let mut uploaded = 0;

    let mut reader_stream = buf.chunks(ONE_MB_SIZE.try_into().unwrap()).map(|e| e.to_vec());
    let stream = async_stream::stream! {
        let state = arc.read().await;
        while let Some(chunk) = reader_stream.next() {
            let new = min(uploaded + (chunk.len() as usize), total_size);
            uploaded = new;
            state.send((new as f32) / (total_size as f32));

            yield Ok(chunk) as Result<Vec<u8>, anyhow::Error>;
        }

        state.send(1 as f32);
        drop(state);
    }; */

    let e = surf::post(url).body(buf).send().await;
    if e.is_err() {
        let err = e.unwrap_err();
        let err: anyhow::Error = err.into_inner();

        return Err(err);
    }

    return Ok(e.unwrap());
}
