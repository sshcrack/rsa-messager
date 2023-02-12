use lazy_static::lazy_static;
use openssl::{hash::MessageDigest, symm::Cipher};

pub const UUID_SIZE: usize = 16;
pub const U64_SIZE: usize = 8;
pub const USIZE_SIZE: usize = 4;
pub const CHUNK_SIZE: u64 = 10 * 1000 * 1000 ; // 10 MB
pub const CHUNK_SIZE_I64: i64 = CHUNK_SIZE as i64 ; // 10 MB

pub const ONE_MB_SIZE: u64 = 1000 * 1000 ; // 1 MB
pub const AES_KEYSIZE_BITS: usize = 256;
pub const AES_KEYSIZE_BYTES: usize = AES_KEYSIZE_BITS / USIZE_SIZE;

pub const RSA_KEY_BITS: u32 = 2048;
pub const RSA_PUBKEY_BYTES: usize = 451;

pub const AES_IVSIZE_BITS: usize = 128;
pub const AES_IVSIZE_BYTES: usize = AES_IVSIZE_BITS / USIZE_SIZE;


lazy_static! {
    pub static ref AES_DIGEST: Cipher = Cipher::aes_256_cbc();
    pub static ref MSG_DIGEST: MessageDigest = MessageDigest:: sha256();
}