use crate::{command::run_as_root, task::pool};
use log::{debug, info};
use repack::compression::{Operation, Upack};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};
use tinyvec::TinyVec;

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

pub(crate) fn extract_tar<D: AsRef<Path>>(
    tar_path: &Path,
    dst_dir: D,
) -> io::Result<()> {
    let dst = dst_dir.as_ref();
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    #[allow(unused_variables)]
    let osstr = OsStr::new;

    run_as_root(
        "tar",
        &[
            osstr("--directory"),
            dst.as_ref(),
            osstr("-xf"),
            tar_path.as_ref(),
        ],
    );

    Ok(())
}

pub(crate) fn pack_tar<S: AsRef<OsStr>>(
    src_dir: S,
    tar_path: &Path,
    exclude_dev: bool,
) -> io::Result<()> {
    #[allow(unused_variables)]
    let osstr = OsStr::new;

    let mut args = TinyVec::<[&OsStr; 24]>::new();

    let src_osdir = src_dir.as_ref();

    // doas tar --posix --directory src_dir --exclude=... -cf tar_path .
    args.extend([
        osstr("--posix"),
        osstr("--directory"),
        src_osdir,
        osstr(r##"--exclude=proc/*"##),
        osstr(r#"--exclude=sys/*"#),
        osstr(r#"--exclude=tmp/*"#),
        osstr(r#"--exclude=var/tmp/*"#),
        osstr(r#"--exclude=run/*"#),
        osstr(r#"--exclude=mnt/*"#),
        osstr(r#"--exclude=media/*"#),
        osstr(r#"--exclude=var/cache/apt/pkgcache.bin"#),
        osstr(r#"--exclude=var/cache/apt/srcpkgcache.bin"#),
        osstr(r#"--exclude=var/cache/apt/archives/*deb"#),
        osstr(r#"--exclude=var/cache/apt/archives/partial/*"#),
        // osstr(r#"--exclude=var/cache/apt/archives/lock"#),
    ]);

    if exclude_dev {
        args.push(osstr(r#"--exclude=dev/*"#));
    }

    args.extend([osstr("-cf"), tar_path.as_ref(), osstr(".")]);

    run_as_root("tar", &args);

    // At least two levels of directories are required to avoid deleting the root directory.
    if Path::new(src_osdir)
        .components()
        .count()
        >= 2
    {
        run_as_root("rm", &[osstr("-rf"), src_osdir]);
    }

    Ok(())
}
