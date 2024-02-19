use crate::{
    command::{force_remove_item_as_root, run_as_root},
    task::pool,
};
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

pub(crate) fn extract_tar_as_root<D: AsRef<Path>>(
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
        true,
    );

    Ok(())
}

/// Invokes the `tar` command as root and packages the `src_dir` to `tar_path`.
pub(crate) fn pack_tar_as_root<S: AsRef<OsStr>>(
    src_dir: S,
    tar_path: &Path,
    exclude_dev: bool,
) {
    let osstr = OsStr::new;

    if let Some(par) = tar_path.parent() {
        if !par.exists() {
            // ignore err
            let _ = fs::create_dir_all(par);
        }
    }

    let mut args = TinyVec::<[&OsStr; 24]>::new();

    let src_osdir = src_dir.as_ref();

    // doas tar --posix --directory src_dir --exclude=... -cf tar_path .
    args.extend(["--posix", "--directory"].map(osstr));
    args.push(src_osdir);
    args.extend(
        [
            r"--exclude=proc/*",
            r"--exclude=sys/*",
            r"--exclude=tmp/*",
            r"--exclude=var/tmp/*",
            r"--exclude=run/*",
            r"--exclude=mnt/*",
            r"--exclude=media/*",
            r"--exclude=boot/*",
            r"--exclude=var/cache/apt/pkgcache.bin",
            r"--exclude=var/cache/apt/srcpkgcache.bin",
            r"--exclude=var/cache/apt/archives/*deb",
            r"--exclude=var/cache/apt/archives/partial/*",
            r"--exclude=var/cache/archive-copier/*",
            r"--exclude=var/lib/apt/lists/*.*",
            // r#"--exclude=var/cache/apt/archives/lock"#,
        ]
        .map(osstr),
    );

    if exclude_dev {
        args.push(osstr(r#"--exclude=dev/*"#));
    }

    args.extend([osstr("-cf"), tar_path.as_ref(), osstr(".")]);

    run_as_root("tar", &args, true);

    let internal_dir = |s| Path::new(src_osdir).join(s);

    let sys_dir = internal_dir("sys");
    let proc_dir = internal_dir("proc");

    if sys_dir.join("kernel").exists() {
        run_as_root("umount", &[osstr("-lf"), sys_dir.as_ref()], false);
        run_as_root("umount", &[osstr("-lf"), proc_dir.as_ref()], false);
    }

    force_remove_item_as_root(src_osdir);
}
