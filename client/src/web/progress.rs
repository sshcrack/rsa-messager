use anyhow::anyhow;
use crossbeam_channel::Sender;
use tokio::sync::RwLock;
use std::{cmp::min, sync::Arc};

use futures_util::StreamExt;
use reqwest::Client;


pub async fn download_file(
    client: &Client,
    url: String,
    sender: Sender<f32>
) -> anyhow::Result<Vec<u8>> {
    let arc = Arc::new(RwLock::new(sender));

    // Reqwest setup
    let res = client
        .get(url.clone())
        .send()
        .await
        .or(Err(anyhow!(format!("Failed to GET from '{}'", &url))))?;
    let total_size = res
        .content_length()
        .ok_or(anyhow!(format!("Failed to get content length from '{}'", &url)))?;

    // download chunks
    let mut buffer = Vec::new();
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    let state = arc.read().await;
    while let Some(item) = stream.next().await {
        if item.is_err() {
            drop(state);
            return Err(item.unwrap_err().into());
        }

        let mut chunk = item?.to_vec();
        buffer.append(&mut chunk);

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
    }

    return Ok(buffer);
}

pub async fn upload_file(
    client: &Client,
    url: String,
    buf: Vec<u8>,
    //sender: Sender<f32>
) -> anyhow::Result<()> {
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

    client
        .post(url)
        .body(buf)
        .send()
        .await?;

    return Ok(());
}
