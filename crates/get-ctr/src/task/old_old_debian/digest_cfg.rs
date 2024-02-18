use super::{deser_ron, TarFile, ZstdOp};
use crate::{
    cfg::digest::{self, DockerMirror, FileMirror},
    docker::{
        get_oci_platform,
        repo::Repository,
        repo_map::{MainRepo, RepoMap},
    },
    task::old_old_debian::{
        docker_task::{
            self, platforms_ron_name, repo_digests_filename, MainRepoDigests,
        },
        BUILD_TIME_RON,
    },
};
use byteunit::ByteUnit;
use ron::{extensions::Extensions, ser::PrettyConfig};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};
use url::Url;

pub(crate) const DISTROS_THAT_REQUIRE_XTERM: [&str; 8] = [
    "bo", "hamm", "slink", "potato", "woody", "sarge", "etch", "warty",
];

// create_digest
pub(crate) fn create_digest_cfg<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
    dst_files: &[PathBuf],
) -> anyhow::Result<()> {
    let mut root_cfg = false;
    let mut digest_os_config = [digest::OS::default(); 1];
    let mut digest_os_tag = Vec::with_capacity(16);

    for r in repos {
        if !root_cfg {
            root_cfg = true;
            init_root_cfg(r, &mut digest_os_config)?;
        }

        let TarFile {
            ref tar_path,
            ref docker_dir,
            ..
        } = r.base_tar_name()?;
        log::debug!("docker_dir: {docker_dir:?}");

        let tar_size = tar_path.metadata()?.len();
        // -----------------------
        let docker = update_docker_cfg(docker_dir, r);
        let tag_name = get_tag_name(docker_dir)?;
        let archive_file = archive_file_cfg(r, docker_dir, tar_size, &tag_name)?;

        let build_time =
            deser_ron::<time::OffsetDateTime, _>(docker_dir.join(BUILD_TIME_RON))?;

        let os_tag = digest::MainTag::builder()
            .name(tag_name)
            .arch(r.get_arch().to_owned())
            .datetime(current_utc(build_time))
            .docker(docker)
            .file(archive_file)
            .build();
        digest_os_tag.push(os_tag);
    }

    digest_os_config[0].tag = digest_os_tag;
    let digest_cfg = digest::Digests::builder()
        .os(digest_os_config)
        .build();

    for p in dst_files {
        create_digest_file(&digest_cfg, p)?;
    }

    Ok(())
}

fn create_digest_file(
    digest_cfg: &digest::Digests,
    dst_file: &Path,
) -> Result<(), anyhow::Error> {
    let yaml =
        || -> serde_yaml::Result<String> { serde_yaml::to_string(&digest_cfg) };

    let create_parent = || -> io::Result<()> {
        match dst_file.parent() {
            Some(p) if p.as_os_str().is_empty() => Ok(()),
            Some(p) if !p.exists() => {
                dbg!(p);
                log::info!(
                    "pwd: {:?}, creating the dir: {p:?}",
                    env::current_dir()?
                );
                fs::create_dir_all(p)
            }
            _ => Ok(()),
        }
    };

    match dst_file.extension() {
        Some(s)
            if s.to_string_lossy()
                .trim()
                .is_empty() =>
        {
            log::info!("Empty extension");
            println!("{}", yaml()?);
        }
        None => {
            log::warn!(
                "Since there is no file extension, the file({dst_file:?}) will not be saved."
            );
            println!("{}", yaml()?)
        }
        Some(ext) if ext == OsStr::new("ron") => {
            create_parent()?;
            let ron = ron::ser::to_string_pretty(
                &digest_cfg,
                PrettyConfig::default()
                    .enumerate_arrays(true)
                    // .depth_limit(1)
                    .extensions(Extensions::IMPLICIT_SOME),
            )?;
            fs::write(dst_file, ron)?;
        }
        _ => {
            create_parent()?;
            fs::write(dst_file, yaml()?)?;
        }
    };
    Ok(())
}

fn get_tag_name(docker_dir: &Path) -> Result<String, ron::de::SpannedError> {
    deser_ron::<String, _>(&docker_dir.join("tag.ron"))
}

fn current_utc(build_time: time::OffsetDateTime) -> digest::DateTime {
    digest::DateTime::builder()
        .build_time(build_time)
        .update_time(time::OffsetDateTime::now_utc())
        .build()
}

fn archive_file_cfg(
    r: &Repository<'_>,
    docker_dir: &Path,
    tar_size: u64,
    tag_name: &str,
) -> Result<digest::ArchiveFile, anyhow::Error> {
    let ZstdOp {
        path: zstd_path,
        lv: zstd_lv,
    } = deser_ron::<ZstdOp<PathBuf>, _>(&docker_dir.join("zstd.ron"))?;

    let zstd_filename = zstd_path
        .file_name()
        .expect("Invalid ZSTD file");
    let lossy_filename = zstd_filename.to_string_lossy();

    let zstd_digests = ["blake3", "sha256"].map(|algo| {
        log::info!("Getting {algo} checksum ..., zstd_path: {zstd_path:?}");
        let hex = match algo {
            "blake3" => hash_digest::blake3::get(&zstd_path),
            _ => hash_digest::sha256::get(&zstd_path),
        }
        .expect("Failed to get hash digest");

        let cmt = match algo {
            "blake3" => {
                format!(
                    r##"Usage:
    # run apt as root (i.e., +sudo/+doas)
    apt install b3sum

    # check blake3 hash
    echo '{hex}  {lossy_filename}' > blake3.txt
    b3sum --check blake3.txt

"##
                )
            }
            _ => {
                format!(
                    r##"Usage:
    # check sha256 hash
    echo '{hex}  {lossy_filename}' > sha256.txt
    sha256sum --check sha256.txt

"##
                )
            }
        };

        digest::HashDigest::builder()
            .algorithm(algo)
            .cmt(cmt)
            .hex(hex.to_string())
            .build()
    });
    let zstd_meta = zstd_path.metadata()?;
    let zstd_size = zstd_meta.len();
    let readable_size = ByteUnit::new(zstd_size);

    let tar_and_zstd_size = ByteUnit::new(tar_size + zstd_size);
    let tar_readable = ByteUnit::new(tar_size);

    let file_size_cmt = format!(
        r#"Ideally:
    zstd size => download size (i.e. Consumes {readable_size} of traffic)
    tar size => uncompressed size (Actually, the extracted content is >= {tar_readable})
    zstd + tar size ~= space occupation for initial installation
        (i.e., Requires at least {tar_and_zstd_size} of disk storage space, but actually needs more)

"#
    );

    let file_size = digest::FileSize::builder()
        .bytes(zstd_size)
        .cmt(file_size_cmt)
        .readable(readable_size)
        .kib((!readable_size.is_kib()).then(|| ByteUnit::new_kib(zstd_size)))
        .mib((!readable_size.is_mib()).then(|| ByteUnit::new_mib(zstd_size)))
        .tar_bytes(tar_size)
        .tar_readable(tar_readable)
        .build();

    let nspawn_env = match r.get_series().as_str() {
        // `-E xx ` retains a space at the end
        s if DISTROS_THAT_REQUIRE_XTERM.contains(&s) => "-E TERM=xterm ",
        _ => "",
    };

    let gh_repo = match r.get_project() {
        &"debian" | &"debian-sid" => "debian-museum",
        // &"ubuntu"
        _ => "ubuntu-museum",
    };

    // github.com/2cd/debian-museum/releases/download
    let gh_url = format!(
        "github.com/{owner}/{gh_repo}/releases/download",
        owner = r.get_owner()
    );

    let zstd_mirror = [("github", gh_url)].map(|(name, u)| {
        let (tag_prefix, tag) = match r.get_tag() {
            Some(t) => ("-", *t),
            None => ("", ""),
        };

        let url_str = format!(
            "https://{u}/{}{tag_prefix}{tag}/{}",
            r.get_version(),
            zstd_filename.to_string_lossy()
        );

        let cmt = format!(
            r##"Usage:
    mkdir -p ./tmp/{tag_name}
    cd tmp
    curl -LO '{url_str}'

    # run gnutar or bsdtar (libarchive-tools) as root (e.g., doas tar -xvf file.tar.zst)
    tar -C {tag_name} -xf {zstd_filename:?}

    # run apt as root (i.e., +sudo/+doas)
    apt install systemd-container qemu-user-static

    # run nspawn as root (i.e., +sudo/+doas)
    systemd-nspawn -D {tag_name} {nspawn_env}-E LANG=$LANG

"##,
        );
        FileMirror::builder()
            .name(name)
            .cmt(cmt)
            .url(Url::parse(&url_str).expect("Invalid URL"))
            .build()
    });
    let archive_file = digest::ArchiveFile::builder()
        .name(zstd_filename)
        .digest(zstd_digests)
        .modified_time(zstd_meta.modified()?)
        .zstd(
            digest::Zstd::builder()
                .level(zstd_lv)
                .build(),
        )
        .size(file_size)
        .mirror(zstd_mirror)
        .build();
    Ok(archive_file)
}

fn update_docker_cfg(docker_dir: &Path, r: &Repository<'_>) -> digest::Docker {
    let docker_mirrors =
        [("ghcr", "ghcr.ron"), ("reg", "reg.ron")].map(|(name, ron)| {
            let repositories = deser_reg_ron(docker_dir.join(ron));

            DockerMirror::builder()
                .name(name)
                .repositories(repositories)
                .build()
        });

    let repo_digest_file = docker_dir.join(repo_digests_filename("ghcr.ron"));
    let docker = digest::Docker::builder()
        .platform(get_oci_platform(r.get_arch()))
        .mirror(docker_mirrors)
        .repo_digests({
            let f = repo_digest_file;
            f.exists()
                .then(|| deser_reg_ron(f))
        })
        .build();
    docker
}

fn deser_reg_ron<P: AsRef<Path>>(reg_ron: P) -> MainRepoDigests {
    deser_ron::<MainRepoDigests, _>(reg_ron).unwrap_or_else(|e| {
        log::error!("Err: {e}");
        panic!("Failed to deser")
    })
}

pub(crate) fn init_root_cfg(
    r: &Repository<'_>,
    digest_os_config: &mut [digest::OS; 1],
) -> Result<(), anyhow::Error> {
    let docker_ron = r.docker_ron_filename();
    let mut ghcr_repos = MainRepoDigests::new();
    let mut reg_repos = MainRepoDigests::new();

    for k in deser_ron::<RepoMap, _>(&docker_ron)?
        .clone()
        .into_keys()
    {
        match k {
            MainRepo::Ghcr(v) => ghcr_repos.push(v),
            MainRepo::Reg(v) => reg_repos.push(v),
        }
    }

    let repo_digest_map = deser_ron::<docker_task::MainRepoDigestMap, _>(
        docker_task::repo_digests_filename(&docker_ron),
    )?;

    let cmt = format!(
        r##"Usage:
    docker run -it --rm {}"##,
        ghcr_repos[0]
    );

    let mirrors = [("ghcr", ghcr_repos), ("reg", reg_repos)].map(|(name, repos)| {
        DockerMirror::builder()
            .name(name)
            .repositories(repos)
            .repo_digests({
                let v = repo_digest_map
                    .get(name)
                    // .cloned()
                    .map(|x| {
                        let mut vec = x.to_vec();
                        vec.sort_unstable();
                        vec.dedup();
                        MainRepoDigests::from_iter(vec)
                    });
                v
            })
            .build()
    });

    let platforms = deser_ron::<Vec<String>, _>(platforms_ron_name(&docker_ron))?;
    let os_docker = digest::Docker::builder()
        .oci_platforms(platforms)
        .mirror(mirrors)
        .cmt(cmt)
        .build();

    digest_os_config[0] = digest::OS::builder()
        .codename(r.get_codename().to_owned())
        .series(r.get_series())
        .name(r.get_osname().to_owned())
        .version(r.get_version().to_owned())
        .docker(os_docker)
        .build();
    Ok(())
}
