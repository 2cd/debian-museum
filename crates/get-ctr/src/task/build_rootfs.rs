use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::bail;
use tinyvec::TinyVec;
pub(crate) const DEB_ENV: &str = "DEBIAN_FRONTEND=noninteractive";

use crate::{
    command::{
        create_dir_all_as_root, force_remove_item_as_root, move_item_as_root,
        run_as_root, run_nspawn,
    },
    docker::repo::{Repository, SrcFormat},
    task::{
        compression::pack_tar_as_root,
        old_old_debian::{TarFile, BUILD_TIME_RON},
    },
};

/// Serializes `now_utc()` to a string and write to `$docker_dir/build-time.ron`
pub(crate) fn create_build_time_ron(docker_dir: &Path) -> anyhow::Result<()> {
    let build_time = ron::to_string(&time::OffsetDateTime::now_utc())?;
    fs::write(docker_dir.join(BUILD_TIME_RON), build_time)?;
    Ok(())
}

pub(crate) fn obtain<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
) -> anyhow::Result<()> {
    #[cfg(not(debug_assertions))]
    let iter = repos;

    #[cfg(debug_assertions)]
    let iter = repos
        .into_iter()
        .filter(|x| x.get_arch() == &"x64");

    for repo in iter {
        log::trace!("{repo:#?}");

        let TarFile {
            // ref tar_fname,
            ref tar_path,
            ref docker_dir,
            ..
        } = repo.base_tar_name()?;

        create_build_time_ron(docker_dir)?;

        let Some(deb_src) = repo.get_debootstrap_src() else {
            bail!("Invalid debootstrap source")
        };
        let rootfs_dir = docker_dir.join("rootfs");

        run_debootstrap(deb_src, repo, &rootfs_dir);

        if let Some(src) = repo.get_source() {
            let mirror_dir = get_mirror_dir_based_on(docker_dir)?;
            src.create_src_list(
                repo.get_series(),
                &mirror_dir,
                *repo.get_components(),
            )?;
            move_mirror_list_to_rootfs(&mirror_dir, &rootfs_dir, *repo.get_deb822())?
        }

        // ubuntu16.04: apt-get purge makedev
        run_nspawn(
            &rootfs_dir,
            "apt-get update && apt-get upgrade -y; apt-get clean; exit 0",
        );

        pack_tar_as_root(&rootfs_dir, tar_path, true);
    }
    Ok(())
}

pub(crate) fn get_mirror_dir_based_on(docker_dir: &Path) -> io::Result<PathBuf> {
    let mirror_dir: PathBuf = docker_dir.join("mirrors");
    if !mirror_dir.exists() {
        fs::create_dir_all(&mirror_dir)?;
    }
    Ok(mirror_dir)
}
fn run_debootstrap(
    deb_src: &crate::cfg::debootstrap::DebootstrapSrc,
    repo: &Repository<'_>,
    rootfs_dir: &std::path::PathBuf,
    // osstr: fn(&str) -> &OsStr,
) {
    let osstr = OsStr::new;
    let mut args = TinyVec::<[&OsStr; 10]>::new();
    args.extend(
        [
            // "--foreign",
            "--no-check-gpg",
            "--components",
            deb_src.get_components(),
            "--arch",
            repo.get_deb_arch()
                .expect("Invalid Debian Architecture"),
        ]
        .map(osstr),
    );

    if !repo.get_no_minbase() {
        args.extend(["--variant", "minbase"].map(osstr))
    }

    if let Some(pkgs) = deb_src.get_include_pkgs() {
        args.extend(["--include", pkgs].map(osstr));
    }

    args.push(osstr(deb_src.get_suite()));
    args.push(rootfs_dir.as_ref());
    args.push(osstr(deb_src.get_url().as_str()));

    // #[cfg(not(debug_assertions))]
    run_as_root("/usr/sbin/debootstrap", &args);
}

/// - if deb822:              mirrors/mirror.sources -> rootfs/etc/apt/sources.list.d/
/// - if one-line-style(not deb822):  mirrors/sources.list -> rootfs/etc/apt/
pub(crate) fn move_mirror_list_to_rootfs(
    mirror_dir: &Path,
    rootfs_dir: &Path,
    deb822: bool,
) -> anyhow::Result<()> {
    let src_list = rootfs_dir.join("etc/apt/sources.list");

    // move: rootfs/etc/apt/sources.list -> rootfs/etc/apt/sources.list.bak
    {
        let src_list_bak = src_list.with_extension("list.bak");
        if src_list.exists() {
            log::debug!("move item: {src_list:?} -> {src_list_bak:?}");
            move_item_as_root(&src_list, src_list_bak);
        }
    }

    // src_list_dir: rootfs/etc/apt/sources.list.d
    let src_list_dir = src_list.with_extension("list.d");
    log::debug!("list.d: {src_list_dir:?}");

    // if deb822: mirrors/mirror.sources -> rootfs/etc/apt/sources.list.d/
    // if legacy: mirrors/sources.list -> rootfs/etc/apt/
    {
        let (src, dst) = match deb822 {
            true => (mirror_dir.join("mirror.sources"), src_list_dir),
            _ => (mirror_dir.join("sources.list"), src_list),
        };
        log::debug!("move item: {src:?} -> {dst:?}");
        move_item_as_root(src, dst)
    }

    // move: mirrors -> rootfs/usr/local/etc/apt/
    {
        let local_dir = rootfs_dir.join("usr/local/etc/apt/mirrors");
        create_dir_all_as_root(&local_dir);
        force_remove_item_as_root(&local_dir);
        move_item_as_root(mirror_dir, local_dir);
    }

    Ok(())
}
