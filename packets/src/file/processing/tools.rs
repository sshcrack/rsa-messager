use crate::consts::CHUNK_SIZE;
use anyhow::anyhow;
use log::trace;


pub fn get_max_threads(size: u64) -> u64 {
    let size_float = size as f64;
    let chunk = CHUNK_SIZE as f64;

    let threads = ((size_float / chunk).ceil()) as u64;
    return threads;
}

pub fn get_chunk_size(chunk_index: u64, size: u64) -> anyhow::Result<u64> {
    let left = size % CHUNK_SIZE;
    let max_threads  =get_max_threads(size);

    if chunk_index > max_threads {
        trace!("Invalid chunk index {} with size {} and max_threads {}", chunk_index, size, max_threads);
        return Err(anyhow!("Invalid chunk index with given size"));
    }

    if chunk_index == max_threads -1 {
        return Ok(left);
    }

    return Ok(CHUNK_SIZE);
}