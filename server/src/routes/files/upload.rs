use std::{collections::HashMap, str::FromStr};

use futures_util::{Stream, StreamExt};
use warp::Buf;


pub async fn on_upload<S, B>(mut body: S) -> Result<Box<dyn warp::Reply>, warp::Rejection>
where
S: Stream<Item = Result<B, warp::Error>> + Send + 'static + Unpin,
B: Buf
{
    while let Some(info) = body.next().await {
    }

    return Ok(Box::new(warp::reply::html("ur moma")));
}