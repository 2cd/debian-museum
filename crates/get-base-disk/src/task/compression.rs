use crate::task::pool;
use log::{debug, info};
use repack::compression::{Operation, Upack};
use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) fn decompress_gzip(gz_fname: &str, tar_path: &Path) -> io::Result<()> {
    let unpack_gz = Upack::new(gz_fname, tar_path);
    info!(
        "Decompressing {:?} to {:?}",
        unpack_gz.source.path, unpack_gz.target.path,
    );
    debug!("operation: {:?}", unpack_gz.operation);
    unpack_gz.run()?;
    Ok(())
}

pub(crate) fn spawn_zstd_thread(
    tar_path: PathBuf,
    zstd_target: PathBuf,
    zstd_lv: Option<&u8>,
) {
    let pool = pool::global_pool();
    let lv = zstd_lv.map_or(Some(19), |x| Some(*x as _));

    pool.execute(move || {
        let zstd = Upack::new(&tar_path, &zstd_target)
            .with_operation(Operation::encode(lv));
        info!(
            "Compressing {:?} to {:?}",
            zstd.source.path, zstd.target.path,
        );
        debug!("operation: {:?}", zstd.operation);
        let _ = zstd.run();
    })
}
