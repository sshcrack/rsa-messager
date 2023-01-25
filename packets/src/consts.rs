use lazy_static::lazy_static;
use openssl::hash::MessageDigest;

pub const UUID_SIZE: usize = 16;
pub const U64_SIZE: usize = 8;
pub const USIZE_SIZE: usize = 8;
pub const CHUNK_SIZE: u64 = 10 * 1000 * 1000 ; // 10 MB
pub const ONE_MB_SIZE: u64 = 1000 * 1000 ; // 10 MB
pub const CHUNK_SIZE_I64: i64 = CHUNK_SIZE as i64 ; // 10 MB

lazy_static! {
    pub static ref MSG_DIGEST: MessageDigest = MessageDigest:: sha256();
}