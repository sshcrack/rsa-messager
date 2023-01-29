use anyhow::anyhow;
use futures_util::{Stream, StreamExt};
use log::trace;
use openssl::{pkey::PKey, rsa::Rsa, sign::Verifier};
use packets::{
    consts::{MSG_DIGEST, U64_SIZE, USIZE_SIZE, UUID_SIZE},
    util::tools::{u64_from_vec, usize_from_vec, uuid_from_vec}, file::processing::ready::ChunkReadyMsg, types::ByteMessage,
};
use tokio::{
    fs::{File, remove_file},
    io::AsyncWriteExt,
};
use warp::{hyper::StatusCode, reply, Buf, ws::Message};

use crate::{
    file::tools::{get_chunk_file, get_uploading_file},
    utils::{
        arcs::get_user,
        stream::{s2vec, vec_from_stream}, tools::send_msg_specific,
    },
};

pub async fn on_upload<S, B>(mut body: S) -> Result<Box<dyn warp::Reply>, warp::Rejection>
where
    S: Stream<Item = Result<B, warp::Error>> + Send + 'static + Unpin,
    B: Buf,
{
    let res: anyhow::Result<()> = async move {
        let mut previous: Vec<u8> = Vec::new();
        let b_signature_size = s2vec(&mut body, USIZE_SIZE, &mut previous).await?;
        let signature_size = usize_from_vec(&mut b_signature_size.clone())?;


        let signature = vec_from_stream(&mut body, signature_size, &mut previous).await?;
        let b_uuid = s2vec(&mut body, UUID_SIZE, &mut previous).await?;
        let uuid = uuid_from_vec(&mut b_uuid.clone())?;

        let b_chunk_index = s2vec(&mut body, U64_SIZE, &mut previous).await?;
        let chunk_index = u64_from_vec(&mut b_chunk_index.clone())?;

        trace!("Getting file in upload {}", uuid);
        let file = get_uploading_file(&uuid).await?;
        let info = get_user(&file.sender).await?;

        let pub_key = info.public_key;
        if pub_key.is_none() {
            return Err(anyhow!("Public key for user is none."));
        }

        let pub_key = pub_key.unwrap();
        let pub_key = Rsa::public_key_from_pem(pub_key.as_bytes())?;
        let p_key = PKey::from_rsa(pub_key.clone())?;

        let file_path = get_chunk_file(&uuid, chunk_index).await?;
        let mut chunk_file = File::create(file_path.clone()).await?;

        let e = std::env::current_dir()?;
        let e = e.join(file_path.clone());
        let e =  e.to_str();

        if e.is_some() {
            trace!("Writing chunk at {}", e.unwrap());
        }

        let inner = async {
            chunk_file.write_all(&b_signature_size).await?;
            chunk_file.write_all(&signature).await?;
            chunk_file.write_all(&b_uuid).await?;
            chunk_file.write_all(&b_chunk_index).await?;
            chunk_file.write_all(&previous).await?;

            let mut verifier = Verifier::new(*MSG_DIGEST, &p_key)?;
            verifier.update(&previous)?;

            trace!("Starting to store file {}...", uuid);
            while let Some(item) = body.next().await {
                let item = item?;
                let item = item.chunk();

                verifier.update(item)?;
                chunk_file.write_all(item).await?;
            }

            let is_valid = verifier.verify(&signature)?;
            if !is_valid {
                return Err(anyhow!("Chunk is not valid."));
            }

            trace!("Sending chunk ready with uuid {}", uuid);
            send_msg_specific(file.receiver, Message::binary(ChunkReadyMsg {
                uuid,
                chunk_index
            }.serialize())).await?;
            Ok(()) as anyhow::Result<()>
        };

        let res = inner.await;
        if res.is_err() {
            let e = chunk_file.shutdown().await;
            remove_file(file_path).await?;
            e?;
            res?;
        }

        Ok(())
    }
    .await;

    if res.is_err() {
        eprintln!("Upload Error: {:?}", res.unwrap_err());
        return Ok(Box::new(reply::with_status(
            "Internal Server Error, (either user request was faulty or a serious bug)",
            StatusCode::INTERNAL_SERVER_ERROR,
        )));
    }

    return Ok(Box::new(warp::reply::html("uploaded.")));
}
