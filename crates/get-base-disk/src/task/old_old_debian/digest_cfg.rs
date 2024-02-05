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

        let TarFile {
            tar_path,
            ref docker_dir,
            ..
        } = r.base_tar_name()?;
        log::debug!("docker_dir: {docker_dir:?}");

        let tar_size = tar_path.metadata()?.len();
        // -----------------------
        let docker = update_docker_cfg(docker_dir, r);
        let archive_file = archive_file_cfg(docker_dir, tar_size, r)?;

        let build_time =
            deser_ron::<time::OffsetDateTime, _>(docker_dir.join(BUILD_TIME_RON))?;

        let os_tag = digest::MainTag::builder()
            .name(get_tag_name(docker_dir)?)
            .arch(r.arch)
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

    // let ron = ron::to_string(&digest_cfg)?;
    // println!("{ron}");

    let yaml = serde_yaml::to_string(&digest_cfg)?;
    println!("{yaml}");

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
    docker_dir: &Path,
    tar_size: u64,
    r: &Repository<'_>,
) -> Result<digest::ArchiveFile, anyhow::Error> {
    let ZstdOp {
        path: zstd_path,
        lv: zstd_lv,
    } = deser_ron::<ZstdOp<PathBuf>, _>(&docker_dir.join("zstd.ron"))?;

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
            let repo_digest_file = docker_dir.join(repo_digests_filename(ron));

            DockerMirror::builder()
                .name(name)
                .repositories(deser_reg_ron(docker_dir.join(ron)))
                .repo_digests(
                    repo_digest_file
                        .exists()
                        .then(|| deser_reg_ron(repo_digest_file)),
                )
                .build()
        });

    let docker = digest::Docker::builder()
        .platform(get_oci_platform(r.arch))
        .mirror(docker_mirrors)
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

    let mirrors = [("ghcr", ghcr_repos), ("reg", reg_repos)].map(|(name, repos)| {
        DockerMirror::builder()
            .name(name)
            .repositories(repos)
            .repo_digests(
                repo_digest_map
                    .get(name)
                    .cloned(),
            )
            .build()
    });

    let platforms = deser_ron::<Vec<String>, _>(platforms_ron_name(&docker_ron))?;
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
