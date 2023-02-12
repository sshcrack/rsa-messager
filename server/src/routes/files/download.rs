use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use log::trace;
use packets::encryption::sign::validate_signature;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;
use warp::{hyper::StatusCode, reply::{self, Response}, http::HeaderValue};

use crate::{
    file::tools::{get_chunk_file, get_uploading_file},
    utils::arcs::get_user
};

pub async fn on_download(
    param: HashMap<String, String>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let uuid = param.get("uuid");
    let index = param.get("index");
    let signature = param.get("signature");

    if uuid.is_none() || index.is_none() || signature.is_none() {
        trace!("UUid, index or signature is none");
        return Ok(Box::new(reply::with_status(
            "Either uuid, index or signatue is null.",
            StatusCode::BAD_REQUEST,
        )));
    }

    let run = || async move {
        let uuid = uuid.unwrap();
        let index = index.unwrap();
        let signature = signature.unwrap();

        let uuid = Uuid::from_str(uuid)?;
        let signature = hex::decode(signature)?;
        let index = u64::from_str(index)?;

        let file = get_uploading_file(&uuid).await?;
        let receiver = get_user(&file.receiver).await?;
        let pub_key = receiver.public_key;

        if pub_key.is_none() {
            trace!("No pubkey");
            return Err(anyhow!("PubKey of receiver is None."));
        }

        let pub_key = pub_key.unwrap();
        let is_valid = validate_signature(&uuid.as_bytes().to_vec(), &signature, &pub_key)?;
        if !is_valid {
            trace!("Not a valid signature");
            return Err(anyhow!("Receiver could not be verified."));
        }

        let chunk_path = get_chunk_file(&uuid, index).await?;

        let size = chunk_path.metadata()?;
        let size = size.len();

        let chunk_file = File::open(&chunk_path).await?;
        let reader = ReaderStream::new(chunk_file);

        trace!("Returning with stream...");
        let body = warp::hyper::Body::wrap_stream(reader);

        let mut resp = warp::reply::Response::new(body);
        let headers = resp.headers_mut();

        headers.insert("Content-Length", HeaderValue::from(size));
        headers.insert("Content-Type", HeaderValue::from_str("application/octet-stream")?);

        println!("Returning with size {}",size);
        return Ok(resp) as anyhow::Result<Response>
    };

    let e = run().await;
    if e.is_err() {
        return Ok(Box::new(reply::with_status(
            "Invalid payload of quest string",
            StatusCode::BAD_REQUEST,
        )));
    }

    return Ok(Box::new(e.unwrap()));
}
