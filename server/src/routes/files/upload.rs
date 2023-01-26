use anyhow::anyhow;
use futures_util::{Stream, StreamExt};
use log::trace;
use openssl::{pkey::PKey, rsa::Rsa, sign::Verifier};
use packets::{
    consts::{MSG_DIGEST, U64_SIZE, USIZE_SIZE, UUID_SIZE},
    util::tools::{u64_from_vec, usize_from_vec, uuid_from_vec},
};
use tokio::{
    fs::{remove_file, File},
    io::AsyncWriteExt,
};
use warp::{hyper::StatusCode, reply, Buf};

use crate::{
    file::tools::{get_chunk_file, get_uploading_file},
    utils::{
        arcs::get_user,
        stream::{s2vec, vec_from_stream},
    },
};

pub async fn on_upload<S, B>(mut body: S) -> Result<Box<dyn warp::Reply>, warp::Rejection>
where
    S: Stream<Item = Result<B, warp::Error>> + Send + 'static + Unpin,
    B: Buf,
{
    let res: anyhow::Result<()> = async move {
        let mut previous: Vec<u8> = Vec::new();
        let mut signature_size = s2vec(&mut body, USIZE_SIZE, &mut previous).await?;
        let signature_size = usize_from_vec(&mut signature_size)?;

        let signature = vec_from_stream(&mut body, signature_size, &mut previous).await?;

        let mut uuid = s2vec(&mut body, UUID_SIZE, &mut previous).await?;
        let uuid = uuid_from_vec(&mut uuid)?;

        let mut chunk_index = s2vec(&mut body, U64_SIZE, &mut previous).await?;
        let chunk_index = u64_from_vec(&mut chunk_index)?;

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
        let mut file = File::create(file_path.clone()).await?;

        let e = std::env::current_dir()?;
        let e = e.join(file_path.clone());
        let e =  e.to_str();

        if e.is_some() {
            trace!("Writing chunk at {}", e.unwrap());
        }

        let temp = pub_key.clone();
        let inner = async {
            file.write_all(&previous).await?;

            let mut verifier = Verifier::new(*MSG_DIGEST, &p_key)?;
            verifier.update(&previous)?;

            while let Some(item) = body.next().await {
                let item = item?;
                let item = item.chunk();

                verifier.update(item)?;
                file.write_all(item).await?;
            }

            let is_valid = verifier.verify(&signature)?;
            if is_valid {
                // TODO remove bc only DEV
                let mut e = File::create("chunks/sig.bin").await?;
                e.write_all(&signature).await?;
                e.shutdown().await?;
                let mut e = File::create("chunks/key.bin").await?;
                e.write_all(&temp.public_key_to_pem()?).await?;
                e.shutdown().await?;
                // --- ------ end
                return Err(anyhow!("Chunk is not valid."));
            }

            Ok(()) as anyhow::Result<()>
        };

        let res = inner.await;
        if res.is_err() {
            let e = file.shutdown().await;
            //TODO add back for release remove_file(file_path).await?;
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
