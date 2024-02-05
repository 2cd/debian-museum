use super::{deser_ron, TarFile, ZstdOp};
use crate::{
    cfg::{
        digest,
        digest::{DockerMirror, FileMirror},
    },
    docker::{
        get_oci_platform,
        repo::{NormalRepos, Repository},
        repo_map::{MainRepo, RepoMap},
    },
};
use byteunit::ByteUnit;
use std::path::{Path, PathBuf};
use url::Url;

// create_digest
pub(crate) fn create_digest<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
) -> anyhow::Result<()> {
    let mut root_cfg = false;
    let mut digest_os_config = [digest::OS::default(); 1];
    let mut digest_os_tag = Vec::with_capacity(16);

    for r in repos {
        if !root_cfg {
            root_cfg = true;
            init_root_cfg(r, &mut digest_os_config)?;
        }

        let TarFile { tar_path, .. } = r.base_tar_name();
        let tar_size = tar_path.metadata()?.len();

        let docker_dir = tar_path
            .parent()
            .expect("Invalid TAR path");
        log::debug!("docker_dir: {docker_dir:?}");

        // -----------------------
        let docker = update_docker_cfg(docker_dir, r);
        let archive_file = archive_file_cfg(docker_dir, tar_size, r)?;

        let os_tag = digest::MainTag::builder()
            .name(get_tag_name(docker_dir)?)
            .arch(r.arch)
            .datetime(current_utc())
            .docker(docker)
            .file(archive_file)
            .build();
        digest_os_tag.push(os_tag);
    }

    digest_os_config[0].tag = digest_os_tag;
    let digest_cfg = digest::Digests::builder()
        .os(digest_os_config)
        .build();

    // let ron = ron::to_string(&digest_cfg)?;
    // println!("{ron}");

    let yaml = serde_yaml::to_string(&digest_cfg)?;
    println!("{yaml}");

    Ok(())
}

fn get_tag_name(docker_dir: &Path) -> Result<String, ron::de::SpannedError> {
    deser_ron::<String, &Path>(&docker_dir.join("tag.ron"))
}

fn current_utc() -> digest::DateTime {
    digest::DateTime::builder()
        .build_time(time::OffsetDateTime::now_utc())
        .build()
}

fn archive_file_cfg(
    docker_dir: &Path,
    tar_size: u64,
    r: &Repository<'_>,
) -> Result<digest::ArchiveFile, anyhow::Error> {
    let ZstdOp {
        path: zstd_path,
        lv: zstd_lv,
    } = deser_ron::<ZstdOp<PathBuf>, &Path>(&docker_dir.join("zstd.ron"))?;

    let zstd_digests = ["blake3", "sha256"].map(|algo| {
        digest::HashDigest::builder()
            .algorithm(algo)
            .hex({
                log::info!("Getting {algo} checksum ..., zstd_path: {zstd_path:?}");
                match algo {
                    "blake3" => hash_digest::blake3::get(&zstd_path),
                    _ => hash_digest::sha256::get(&zstd_path),
                }
                .expect("Failed to get hash digest")
                .to_string()
            })
            .build()
    });
    let zstd_meta = zstd_path.metadata()?;
    let zstd_size = zstd_meta.len();
    let readable_size = ByteUnit::new(zstd_size);
    let file_size = digest::FileSize::builder()
        .bytes(zstd_size)
        .readable(readable_size)
        .kib((!readable_size.is_kib()).then(|| ByteUnit::new_kib(zstd_size)))
        .mib((!readable_size.is_mib()).then(|| ByteUnit::new_mib(zstd_size)))
        .tar_bytes(tar_size)
        .tar_readable(ByteUnit::new(tar_size))
        .build();
    let zstd_filename = zstd_path
        .file_name()
        .expect("Invalid ZSTD file");
    let zstd_mirror = [("github", "github.com/2cd/debian-museum/releases/download")]
        .map(|(name, u)| {
            FileMirror::builder()
                .name(name)
                .url(
                    Url::parse(&format!(
                        "https://{u}/{}/{}",
                        r.version,
                        zstd_filename.to_string_lossy()
                    ))
                    .expect("Invalid URL"),
                )
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
            DockerMirror::builder()
                // .repo_digest(repo_digest)
                .name(name)
                .repositories(
                    deser_ron::<NormalRepos, &Path>(&docker_dir.join(ron))
                        .unwrap_or_else(|e| {
                            log::error!("Err: {e}");
                            panic!("Failed to deser : {ron:?}")
                        })
                        .to_vec(),
                )
                .build()
        });

    let docker = digest::Docker::builder()
        .platform(get_oci_platform(r.arch))
        .mirror(docker_mirrors)
        .build();
    docker
}

pub(crate) fn init_root_cfg(
    r: &Repository<'_>,
    digest_os_config: &mut [digest::OS; 1],
) -> Result<(), anyhow::Error> {
    let docker_ron = r.docker_ron_filename();
    let mut ghcr_repos = Vec::with_capacity(4);
    let mut reg_repos = Vec::with_capacity(4);
    for (k, _) in deser_ron::<RepoMap, &str>(&docker_ron)?.clone() {
        match k {
            MainRepo::Ghcr(v) => ghcr_repos.push(v),
            MainRepo::Reg(v) => reg_repos.push(v),
        }
    }
    let mirrors = [("ghcr", ghcr_repos), ("reg", reg_repos)].map(|(name, repos)| {
        DockerMirror::builder()
            .name(name)
            .repositories(repos)
            .build()
    });

    let platforms =
        deser_ron::<Vec<String>, &str>(docker_ron.trim_end_matches("on"))?;
    let os_docker = digest::Docker::builder()
        .oci_platforms(platforms)
        .mirror(mirrors)
        .build();
    digest_os_config[0] = digest::OS::builder()
        .codename(&r.codename)
        .name(r.project)
        .version(r.version)
        .docker(os_docker)
        .build();
    Ok(())
}
