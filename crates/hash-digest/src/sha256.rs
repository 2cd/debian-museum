use crate::HexStr64;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

pub fn get<P: AsRef<Path>>(path: P) -> io::Result<HexStr64> {
    let mut hasher = Sha256::new();
    let mut buf = BufReader::with_capacity(64 * 1024, File::open(path)?);

    io::copy(&mut buf, &mut hasher)?;
    let hash = hasher.finalize();

    let hex = blake3::Hash::from_bytes(hash.into()).to_hex();
    Ok(hex)
}
