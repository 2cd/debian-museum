use crate::HexStr64;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

pub fn get<P: AsRef<Path>>(path: P) -> io::Result<HexStr64> {
    let path = path.as_ref();
    let mut hasher = blake3::Hasher::new();

    const KIB_256: u64 = 256 * 1024;
    const MIB_16: u64 = 16 * 1024 * 1024;

    match path.metadata()?.len() {
        n if n < KIB_256 => {
            io::copy(&mut File::open(path)?, &mut hasher)?;
        }
        KIB_256..=MIB_16 => {
            const SIZE: usize = 64 * 1024;
            let mut reader = BufReader::with_capacity(SIZE, File::open(path)?);

            let mut buf = [0; SIZE];
            while let Ok(n @ 1..) = reader.read(&mut buf) {
                hasher.update_rayon(unsafe { buf.get_unchecked(..n) });
            }
        }
        _ => {
            hasher.update_mmap_rayon(path)?;
        }
    }

    let hash = hasher.finalize();
    let hex = hash.to_hex();
    // println!("{hex:?}");

    Ok(hex)
}
