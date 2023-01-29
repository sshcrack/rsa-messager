use anyhow::anyhow;
use futures_util::{Stream, StreamExt};
use warp::Buf;

// Short version of vec_from_stream method
pub async fn s2vec<S, B>(stream: &mut S, size: usize, previous: &mut Vec<u8>) -> anyhow::Result<Vec<u8>>
where
S: Stream<Item = Result<B, warp::Error>> + Send + 'static + Unpin,
B: Buf {
    return vec_from_stream(stream, size, previous).await;
}


pub async fn vec_from_stream<S, B>(stream: &mut S, size: usize, previous: &mut Vec<u8>) -> anyhow::Result<Vec<u8>>
where
S: Stream<Item = Result<B, warp::Error>> + Send + 'static + Unpin,
B: Buf
{
    println!("Prev: {} size: {}", previous.len(), size);
    if previous.len() >= size {
        let res: Vec<u8> = previous.splice(0..size, vec![]).collect();
        return Ok(res);
    }

    let mut res: Vec<u8> = previous.clone();
    previous.clear();

    while let Some(item) = stream.next().await {
        let mut item = item?;
        while item.has_remaining() {
            let val = item.get_u8();
            if res.len() < size { res.push(val); }
            else { previous.push(val); }
        }

        if  res.len() == size {
            break;
        }

        if res.len() > size {
            panic!("What the fuck just happened. The result of vec_from_stream should NEVER be higher than the given size");
        }
    }

    if res.len() < size {
        return Err(anyhow!("Error, stream was not long enough."));
    }

    return Ok(res);
}
