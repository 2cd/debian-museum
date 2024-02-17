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
        create_dir_all_as_root, force_remove_item_as_root, move_item_as_root, run,
        run_as_root, run_nspawn,
    },
    docker::repo::Repository,
    task::{
        compression::{extract_tar_as_root, pack_tar_as_root},
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
        // .filter(|x| matches!(x.get_arch(), &"x64" | &"x86"))
        // .filter(|x| matches!(x.get_arch(), &"x86"))
        // dbg
        ;

    const OLD_AMD64: [&str; 19] = [
        "breezy", "dapper", "edgy", "etch", "feisty", "hardy", "hoary", "intrepid",
        "jaunty", "karmic", "lenny", "lucid", "maverick", "natty", "oneiric",
        "sarge", "squeeze", "warty", "wheezy",
    ];

    for repo in iter {
        log::debug!("building: {} ({})", repo.get_codename(), repo.get_version());

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
        let series = repo.get_series().as_str();
        let deb_arch = repo.get_deb_arch();

        // #[cfg(not(debug_assertions))]
        if !rootfs_dir.exists() || !tar_path.exists() {
            match (deb_arch, series) {
                (Some(arch @ "amd64"), s) if OLD_AMD64.contains(&s) => {
                    get_old_rootfs(docker_dir, &rootfs_dir, arch, s)?
                }
                (Some(arch), s)
                    if ["i386", "powerpc", "sparc"].contains(arch)
                        && ["warty", "hoary"].contains(&s) =>
                {
                    get_old_rootfs(docker_dir, &rootfs_dir, arch, s)?
                }
                _ => {
                    let mut ex_pkgs = TinyVec::<[&str; 1]>::new();
                    match series {
                        "squeeze" | "lenny" | "etch" => {
                            ex_pkgs.push("apt-transport-https");
                        }
                        _ => {}
                    };
                    run_debootstrap(deb_src, repo, &rootfs_dir, &ex_pkgs)
                }
            }
        }

        if let Some(src) = repo.get_source() {
            let mirror_dir = get_mirror_dir_based_on(docker_dir)?;
            src.create_src_list(
                repo.get_series(),
                &mirror_dir,
                *repo.get_components(),
            )?;
            move_mirror_list_to_rootfs(&mirror_dir, &rootfs_dir, *repo.get_deb822())?
        }

        patch_deb_rootfs(&rootfs_dir, repo);

        pack_tar_as_root(&rootfs_dir, tar_path, true);
    }
    Ok(())
}

fn get_old_rootfs(
    docker_dir: &Path,
    rootfs_dir: &Path,
    arch: &str,
    series: &str,
) -> Result<(), anyhow::Error> {
    get_rootfs_from_docker(
        &format!("reg.tmoe.me:2096/rootfs/{series}:{arch}",),
        docker_dir,
    );
    let base_tar = docker_dir.join("base.tar");
    extract_tar_as_root(&base_tar, rootfs_dir)?;
    force_remove_item_as_root(base_tar);
    Ok(())
}

// docker run -t --rm -v $docker_dir:/app reg.tmoe.me:2096/rootfs/sarge:amd64 mv base.tar /app
fn get_rootfs_from_docker(docker_repo: &str, docker_dir: &Path) {
    run(
        "docker",
        &[
            "run",
            "-t",
            "--rm",
            "-v",
            &format!("{}:/app", docker_dir.to_string_lossy()),
            docker_repo,
            "mv",
            "base.tar",
            "/app",
        ],
    )
}

fn patch_deb_rootfs(rootfs_dir: &PathBuf, repo: &Repository<'_>) {
    // TODO: fix ubuntu16.04: apt-get purge makedev

    run_nspawn(rootfs_dir, "apt-get update; exit 0");
    dbg!(repo.get_codename());

    // # debian-etch: +debian-backports-keyring
    match repo.get_series().as_str() {
        "etch" | "lenny" => {
            run_nspawn(
                rootfs_dir,
                "apt-get install --assume-yes --force-yes debian-backports-keyring \
                    ;  exit 0",
            );
        }
        _ => {}
    }

    // run_nspawn(
    //     rootfs_dir,
    //     "apt-get install --assume-yes --force-yes locales \
    //         ; for i in C en_US zh_CN; do \
    //             localedef \
    //                 --force \
    //                 --inputfile $i \
    //                 --charmap UTF-8 \
    //                 $i.UTF-8 \
    //         ; done \
    //         ; apt-get purge --assume-yes --force-yes locales \
    //         ; exit 0",
    // );

    run_nspawn(
        rootfs_dir,
        "apt-get dist-upgrade --assume-yes --force-yes \
            ;  for i in apt-utils eatmydata; do \
                    apt-get install --assume-yes --force-yes $i \
            ;  done \
            ;  apt-get clean \
            ;  exit 0",
    );
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
    exclude_pkgs: &[&str],
) {
    let osstr = OsStr::new;
    let mut args = TinyVec::<[&OsStr; 10]>::new();
    let mut ex_packages_arr = TinyVec::<[&str; 24]>::new();

    ex_packages_arr.extend([
        "postfix",
        "postfix-tls",
        "ubuntu-base",
        "popularity-contest",
        "vim",
        "vim-common",
        "wireless-tools",
        "ppp",
        "pppoe",
        "pppconfig",
        "pppoeconf",
        "w3m",
        "kbd",
        "udev",
        "man-db",
    ]);

    if !exclude_pkgs
        .first()
        .is_some_and(|x| x.is_empty())
    {
        ex_packages_arr.extend(exclude_pkgs.iter().copied())
    }

    let ex_pkgs_comma_str = ex_packages_arr.join(",");

    args.extend(
        [
            "--no-check-gpg",
            "--exclude",
            &ex_pkgs_comma_str,
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

    // if deb822: mirrors/mirror.sources -> rootfs/etc/apt/sources.list.d/mirror.sources
    // if legacy: mirrors/sources.list -> rootfs/etc/apt/sources.list
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
