use anyhow::anyhow;
use bytes::Bytes;
use futures::{io::BufReader, stream};
use futures::{AsyncBufReadExt, TryStreamExt};
use futures_util::StreamExt;
use log::warn;
use packets::consts::ONE_MB_SIZE;
use std::{cmp::min, sync::Arc};
use surf::Body;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tokio::sync::RwLock as TokioRwLock;

use crate::file::tools::WorkerProgress;

pub async fn download_file(
    url: String,
    sender: &UnboundedSender<WorkerProgress>,
    chunk_index: u64,
) -> anyhow::Result<Vec<u8>> {
    let arc = Arc::new(TokioRwLock::new(sender));

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
    let total_size = total_size.to_string().parse::<u64>()?;

    let mut buffer = Vec::new();
    let stream = BufReader::new(res);
    let arc_stream = Arc::new(TokioRwLock::new(stream));

    let mut downloaded: u64 = 0;

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

        let length = chunk.len();
        buffer.append(&mut chunk);

        let new = min(downloaded + (length as u64), total_size);
        let prog = (new as f32) / (total_size as f32);
        downloaded = new;

        let state = arc.read().await;
        let e = state.send(WorkerProgress {
            progress: prog,
            chunk: chunk_index,
        });
        drop(state);
        e?;
    }

    let state = arc.read().await;
    let e = state.send(WorkerProgress {
        progress: 1 as f32,
        chunk: chunk_index,
    });
    drop(state);
    e?;

    return Ok(buffer);
}

pub async fn upload_file(
    url: String,
    buf: Vec<u8>,
    sender_arc: Arc<RwLock<UnboundedSender<WorkerProgress>>>,
    offset: f32,
    index: u64,
) -> anyhow::Result<surf::Response> {
    let uploaded = Arc::new(RwLock::new(0));

    let e = buf.clone();
    let chunk_size = buf.len();

    let mut cursor = stream::iter(e).chunks(std::cmp::min(ONE_MB_SIZE as usize, buf.len()));

    let sender = sender_arc.write_owned().await;

    let stream = async_stream::stream! {
        while let Some(chunk) = cursor.next().await {
            let l = chunk.len();
            yield Ok(Bytes::from(chunk)) as Result<Bytes, std::io::Error>;

            let mut s = uploaded.write().await;
            *s += l;

            let curr = s.clone();
            drop(s);


            let prog = ((curr as f32) / (chunk_size as f32) * offset ) + offset;

            let e = sender.send(WorkerProgress { chunk: index, progress: prog });
            if e.is_err() {
                warn!("Could not update progress bar (send): {}", e.unwrap_err());
                return;
            }

        }

        drop(sender);
    };

    let reader = Box::pin(stream.into_async_read());
    let e = surf::post(url)
        .body(Body::from_reader(reader, Some(buf.len())))
        .send()
        .await;

    if e.is_err() {
        let err = e.unwrap_err();

        return Err(anyhow::anyhow!(err));
    }

    return Ok(e.unwrap());
}
