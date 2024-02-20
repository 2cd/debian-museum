use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::bail;
use serde::Deserialize;
use tinyvec::TinyVec;
use url::Url;
pub(crate) const DEB_ENV: &str = "DEBIAN_FRONTEND=noninteractive";

use crate::{
    cfg::debootstrap,
    command::{
        create_dir_all_as_root, force_remove_item_as_root, move_item_as_root, run,
        run_and_get_stdout, run_as_root, run_nspawn,
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
    const OLD_AMD64: [&str; 19] = [
        "breezy", "dapper", "edgy", "etch", "feisty", "hardy", "hoary", "intrepid",
        "jaunty", "karmic", "lenny", "lucid", "maverick", "natty", "oneiric",
        "sarge", "squeeze", "warty", "wheezy",
    ];

    for repo in repos.into_iter()
    // .filter(|x| matches!(x.get_arch(), &"x64" | &"x86"))
    // .filter(|x| matches!(x.get_arch(), &"x86"))
    {
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

        let get_rootfs = |arch, series| -> Result<(), anyhow::Error> {
            get_old_rootfs(docker_dir, &rootfs_dir, arch, series)
        };
        const JESSIE_NO_LTS_ARCHS: [&str; 6] =
            ["arm64", "mipsel", "mips", "powerpc", "ppc64el", "s390x"];

        // #[cfg(not(debug_assertions))]
        if !rootfs_dir.exists() || !tar_path.exists() {
            match (deb_arch, series) {
                (Some(arch @ "amd64"), s) if OLD_AMD64.contains(&s) => {
                    get_rootfs(arch, s)?
                }
                // (Some(arch @ "loong64"), s @ "sid") => get_rootfs(arch, s)?,
                (Some(arch), s)
                    if ["warty", "hoary", "gutsy", "potato", "woody"]
                        .contains(&s) =>
                {
                    get_rootfs(arch, s)?
                }
                (Some(arch), s @ "jessie") if JESSIE_NO_LTS_ARCHS.contains(arch) => {
                    get_rootfs(arch, s)?
                }
                (Some(arch), s @ "sarge")
                    if ["mips", "mipsel", "powerpc"].contains(arch) =>
                {
                    get_rootfs(arch, s)?
                }
                _ => {
                    let mut ex_pkgs = TinyVec::<[&str; 1]>::new();
                    match series {
                        "squeeze" | "lenny" | "etch" => {
                            ex_pkgs.push("apt-transport-https");
                        }
                        _ => {}
                    };
                    run_debootstrap(deb_src, repo, &rootfs_dir, &ex_pkgs)?;
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
        &format!("{uri}/rootfs/{series}:{arch}", uri = Repository::REG_URI),
        docker_dir,
    );
    let base_tar = docker_dir.join("base.tar");
    extract_tar_as_root(&base_tar, rootfs_dir)?;
    force_remove_item_as_root(base_tar);
    Ok(())
}

// docker run -t --rm -v $docker_dir:/app reg.tmoe.me:2096/rootfs/sarge:amd64 mv base.tar /app
fn get_rootfs_from_docker(docker_repo: &str, docker_dir: &Path) {
    let args = [
        "run",
        "--platform=linux/amd64",
        "-t",
        "--rm",
        "-v",
        &format!(
            "{}:/host",
            docker_dir
                .canonicalize()
                .expect("Invalid docker dir")
                .to_string_lossy()
        ),
        "--pull",
        "always",
        docker_repo,
        "mv",
        "-f",
        "/base.tar",
        "/host",
    ];
    log::info!("cmd: docker, args: {args:?}");
    run("docker", &args, true);
}

fn patch_deb_rootfs(rootfs_dir: &Path, repo: &Repository<'_>) {
    let series = repo.get_series().as_str();
    let year = match repo.get_title_date() {
        Some(s) => s
            .split('-')
            .next()
            .and_then(|y| y.parse::<u64>().ok())
            .unwrap_or(2020),
        None => 2020,
    };

    log::debug!(
        "codename: {}, arch: {}",
        repo.get_codename(),
        repo.get_deb_arch()
            .unwrap_or("Unknown")
    );
    run_nspawn(rootfs_dir, "apt-get update", false, &["LANG=C.UTF-8"]);

    // # debian-etch: +debian-backports-keyring
    match series {
        "etch" | "lenny" => {
            run_nspawn(
                rootfs_dir,
                "apt-get install --assume-yes --force-yes debian-backports-keyring",
                false,
                &["LANG=C"],
            );
        }
        "xenial" => {
            run_nspawn(
                rootfs_dir,
                // "sed '1a exit 0' -i /var/lib/dpkg/info/makedev.postinst",
                "apt-get autoremove --purge --assume-yes makedev",
                false,
                &[""],
            );
        }
        _ => {}
    }

    let apt_arg = if year >= 2010 { "" } else { "--force-yes" };

    // apt-get install --assume-yes {apt_arg} locales

    if rootfs_dir
        .join("usr/share/i18n/locales/en_US")
        .exists()
    {
        run_nspawn(
            rootfs_dir,
            "
            for i in en_US zh_CN; do
                localedef \
                    --force \
                    --inputfile $i \
                    --charmap UTF-8 \
                    $i.UTF-8
            done
            ",
            false,
            &["LANG=C"],
        );
    }

    run_nspawn(
        rootfs_dir,
        format!(
            "apt-get dist-upgrade --assume-yes {apt_arg} ;
            for i in gpgv apt-utils eatmydata whiptail; do
                apt-get install --assume-yes {apt_arg} $i
            done
            apt-get clean"
        ),
        false,
        &[
            "LANG=C",
            // "container=lxc",
        ],
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
) -> anyhow::Result<()> {
    let osstr = OsStr::new;
    let mut args = TinyVec::<[&OsStr; 10]>::new();
    let mut ex_packages_arr = TinyVec::<[&str; 32]>::new();

    {
        // ubuntu-minimal:
        // adduser, alsa-base, alsa-utils, apt, apt-utils, aptitude, base-files, base-passwd, bash, bsdutils, bzip2, console-setup, console-tools, coreutils, dash, debconf, debianutils, dhcp3-client, diff, dpkg, e2fsprogs, eject, ethtool, findutils, gettext-base, gnupg, grep, gzip, hostname, ifupdown, initramfs-tools, iproute, iputils-ping, less, libc6-i686, libfribidi0, locales, login, lsb-release, makedev, mawk, mii-diag, mktemp, module-init-tools, mount, ncurses-base, ncurses-bin, net-tools, netbase, netcat, ntpdate, passwd, pciutils, pcmciautils, perl-base, procps, python, python-minimal, sed, startup-tasks, sudo, sysklogd, system-services, tar, tasksel, tzdata, ubuntu-keyring, udev, upstart, upstart-compat-sysv, upstart-logd, usbutils, util-linux, util-linux-locales, vim-tiny, wireless-tools, wpasupplicant

        let pkgs = [
            // aptitude search '?priority(required)'
            // aptitude search '?priority(important)'
            // Packages with `important` priority can be excluded.
            //
            "ubuntu-minimal",
            "ubuntu-base",
            "cpio",
            "dmidecode",
            "fdisk",
            "ifupdown",
            "iproute2",
            "iputils-ping",
            "isc-dhcp-common",
            "isc-dhcp-client",
            "kmod",
            "less",
            "logrotate",
            "nano",
            "nftables",
            "procps",
            "udev",
            "vim",
            "vim-common",
            "vim-tiny",
            "udev",
            "man-db",
            "tasksel",
            "tasksel-data",
            // "makedev",
        ];
        ex_packages_arr.extend(pkgs);
    }

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

    let suite = deb_src.get_suite().as_str();

    let real_name = match suite {
        "devel" => get_the_real_name_of_ubuntu_devel(deb_src.get_url()),
        _ => suite,
    };

    let os_name = repo.get_osname();
    fix_script_link(real_name, os_name)?;

    args.push(osstr(real_name));

    args.push(rootfs_dir.as_ref());
    args.push(osstr(deb_src.get_url().as_str()));

    run_as_root("/usr/sbin/debootstrap", &args, true);

    let log_file = rootfs_dir.join("debootstrap/debootstrap.log");

    if log_file.exists() {
        log::debug!(
            "log_file: {}, log_path: {log_file:?}",
            fs::read_to_string(&log_file).unwrap_or_default()
        );
        panic!(
            "Failed to build: {} (dir: {rootfs_dir:?}) with debootstrap",
            repo.get_series()
        )
    };

    Ok(())
}

/// Release File Sample:
///
/// ```
/// Archive: noble
/// Version: 24.04
/// Component: main
/// Origin: Ubuntu
/// Label: Ubuntu
/// Architecture: source
/// ```
#[derive(Deserialize, Debug)]
struct ReleaseUUU {
    #[serde(rename = "Archive")]
    archive: String,
}

/// When using devel as `suite`, the build may fail. So we need to get its real name, e.g., nobel
fn get_the_real_name_of_ubuntu_devel(url: &Url) -> &'static str {
    static N: OnceLock<String> = OnceLock::new();

    N.get_or_init(|| {
        // ubuntu/dists/devel/main/source/Release
        let release_file_url = url
            .join("dists/devel/main/source/Release")
            .expect("Failed to join release url");

        let out = run_and_get_stdout("curl", &["-L", release_file_url.as_str()])
            .expect("Failed to get the real name of ubuntu devel suite");

        serde_yaml::from_str::<ReleaseUUU>(&out)
            .expect("Failed to deser the main/source/Release as yaml")
            .archive
    })
}

/// If the script file does not exist in either "/usr/share/debootstrap/scripts/" or 'env::var_os("DEBOOTSTRAP_DIR")/scripts' then the corresponding symbolic link will be created.
fn fix_script_link(suite: &str, os_name: &str) -> Result<(), io::Error> {
    let env_script_exists =
        get_debootstrap_script_dir_env().is_some_and(|x| x.join(suite).exists());

    let script = Path::new(debootstrap::SCRIPT_DIR).join(suite);

    if !env_script_exists && !script.exists() {
        let src = match os_name {
            "Ubuntu" => "gutsy",
            // &"Debian" => "sid",
            _ => "sid",
        };
        log::info!("Creating the symlink:\t src: {src}, dst: {suite}");
        std::os::unix::fs::symlink(src, suite)?;
        move_item_as_root(suite, script);
    }

    Ok(())
}

fn get_debootstrap_script_dir_env() -> Option<&'static Path> {
    static D: OnceLock<Option<PathBuf>> = OnceLock::new();
    D.get_or_init(|| {
        env::var_os("DEBOOTSTRAP_DIR").map(|x| PathBuf::from(x).join("scripts"))
    })
    .as_deref()
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
