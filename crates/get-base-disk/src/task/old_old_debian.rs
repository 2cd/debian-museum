use byteunit::ByteUnit;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use crate::{
    cfg::digest::{self, DockerMirror, FileMirror},
    command::run_curl,
    docker::{
        get_oci_platform,
        repo::{NormalRepos, Repository},
        repo_map::{MainRepo, RepoMap},
    },
    task::{
        compression::{decompress_gzip, spawn_zstd_thread},
        docker::run_docker_build,
        file::create_docker_file,
        pool::wait_process,
    },
};
use std::{
    collections::BTreeSet,
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

pub(crate) const VERS: [&str; 4] = ["1.3", "2.0", "2.1", "2.2"];

pub(crate) struct TarFile {
    // base: String,
    pub(crate) tar_fname: String,
    pub(crate) tar_path: PathBuf,
}

impl<'r> Repository<'r> {
    fn generate_tar_path(base: &str, tar_fname: &str) -> PathBuf {
        PathBuf::from_iter([base, "docker", tar_fname])
    }

    fn docker_ron_filename(&self) -> String {
        let suffix = self.opt_tag_suffix();
        format!("{}-{}{}.ron", self.version, self.codename, suffix)
    }

    fn base_tar_name(&self) -> TarFile {
        let base: String = self.base_name();
        let tar_fname = format!("{base}.tar");

        let tar_path = Self::generate_tar_path(&base, &tar_fname);
        log::debug!("tar_path: {tar_path:?}");

        // create dir
        {
            log::debug!("creating the tar_path.parent() dir");
            fs::create_dir_all(
                tar_path
                    .parent()
                    .expect("Invalid tar parent"),
            )
            .expect("Failed to create the tar path");
        }

        TarFile {
            tar_fname,
            tar_path,
        }
    }
}

pub(crate) fn obtain<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
) -> io::Result<()> {
    for r in repos {
        log::trace!("{r:#?}");

        let TarFile {
            tar_fname,
            tar_path,
            ..
        } = r.base_tar_name();

        let gz_fname = tar_fname.replace("tar", "tgz");

        // curl
        {
            log::debug!("running curl ...");
            run_curl(
                r.url
                    .as_ref()
                    .expect("Empty URL"),
                &gz_fname,
            );
        }
        // gz
        decompress_gzip(&gz_fname, &tar_path)?;
    }
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ZstdOp<P: AsRef<Path>> {
    path: P,
    lv: u8,
}

pub(crate) fn repack<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
    zstd_lv: Option<&u8>,
) -> anyhow::Result<()> {
    for r in repos {
        log::trace!("{r:#?}");

        let TarFile {
            tar_fname,
            tar_path,
            ..
        } = r.base_tar_name();

        let zstd_file = Path::new("zstd").join(tar_fname.replace("tar", "tar.zst"));
        log::debug!("zstd_file: {zstd_file:?}");

        // create dir
        let op = ZstdOp {
            path: &zstd_file,
            lv: *zstd_lv.unwrap_or(&19),
        };
        {
            log::debug!("creating the zstd_file.parent() dir");
            fs::create_dir_all(
                zstd_file
                    .parent()
                    .expect("Invalid ZSTD-file path"),
            )?;
            fs::write(
                tar_path
                    .parent()
                    .expect("Invalid Tar Path")
                    .join("zstd.ron"),
                ron::to_string(&op)?,
            )?;
        }

        // compress to zstd
        spawn_zstd_thread(tar_path, zstd_file, zstd_lv);
    }
    Ok(())
}

// docker_build
pub(crate) fn docker_build<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
) -> anyhow::Result<()> {
    let mut children = Vec::with_capacity(32);
    let mut tag_map = RepoMap::default();

    let mut docker_ron_name = String::with_capacity(64);
    let mut treeset = BTreeSet::new();

    for r in repos
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            if i == 0 {
                docker_ron_name = r.docker_ron_filename();
            }
            r
        })
    {
        let tar = r.base_tar_name();
        let docker_file = create_docker_file(&tar)?;
        let docker_dir = docker_file
            .parent()
            .expect("Invalid docker dir");
        run_docker_build(r, &mut children, docker_dir, &mut tag_map)?;
        treeset.insert(r.oci_platform());
    }

    // tag_map => docker-ron
    // tree_set => docker-r
    {
        fs::write(&docker_ron_name, ron::to_string(&tag_map)?)?;
        fs::write(
            docker_ron_name.trim_end_matches("on"),
            ron::to_string(&treeset)?,
        )?;
    }

    log::debug!("map: {tag_map:?}");
    wait_process(children);

    Ok(())
}

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
            let docker_ron = r.docker_ron_filename();

            let mut ghcr_repos = Vec::with_capacity(4);
            let mut reg_repos = Vec::with_capacity(4);

            for (k, _) in deser_ron::<RepoMap, &str>(&docker_ron)?.clone() {
                match k {
                    MainRepo::Ghcr(v) => ghcr_repos.push(v),
                    MainRepo::Reg(v) => reg_repos.push(v),
                }
            }

            let mirrors =
                [("ghcr", ghcr_repos), ("reg", reg_repos)].map(|(name, repos)| {
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
        }

        let TarFile { tar_path, .. } = r.base_tar_name();

        let docker_dir = tar_path
            .parent()
            .expect("Invalid TAR path");
        log::debug!("docker_dir: {docker_dir:?}");

        // -----------------------
        let datetime = digest::DateTime::builder()
            .build_time(time::OffsetDateTime::now_utc())
            .build();

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

        let ZstdOp {
            path: zstd_path,
            lv: zstd_lv,
        } = deser_ron::<ZstdOp<PathBuf>, &Path>(&docker_dir.join("zstd.ron"))?;

        let zstd_digests = ["blake3", "sha256"].map(|name| {
            digest::HashDigest::builder()
                .algorithm(name)
                .hex(
                    match name {
                        "blake3" => hash_digest::blake3::get(&zstd_path),
                        _ => hash_digest::sha256::get(&zstd_path),
                    }
                    .expect("Failed to get hash digest")
                    .to_string(),
                )
                .build()
        });

        let zstd_meta = zstd_path.metadata()?;
        let zstd_size = zstd_meta.len();
        let tar_size = tar_path.metadata()?.len();

        let file_size = digest::FileSize::builder()
            .bytes(zstd_size)
            .readable(ByteUnit::new_mib(zstd_size))
            .readable_kib(ByteUnit::new_kib(zstd_size))
            .tar_bytes(tar_size)
            .tar_readable(ByteUnit::new_mib(tar_size))
            .build();

        let zstd_filename = zstd_path
            .file_name()
            .expect("Invalid ZSTD file");

        // https://github.com/2cd/debian-museum/releases/download/1.2/1.2_Rex_i386_base_1996-12-08_XXH3-97d99e29653390c0.tar.zst
        let zstd_mirror =
            [("github", "github.com/2cd/debian-museum/releases/download")].map(
                |(name, u)| {
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
                },
            );

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

        let os_tag = digest::MainTag::builder()
            .name(deser_ron::<String, &Path>(&docker_dir.join("tag.ron"))?)
            .arch(r.arch)
            .datetime(datetime)
            .docker(docker)
            .file(archive_file)
            .build();
        digest_os_tag.push(os_tag);
        // -----------------------
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

fn deser_ron<T, P: AsRef<Path>>(path: P) -> Result<T, ron::de::SpannedError>
where
    T: DeserializeOwned,
{
    log::trace!("pwd: {:?}", env::current_dir()?);
    log::debug!("ron file path: {:?}", path.as_ref());
    ron::de::from_reader(File::open(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dir::set_static_workdir;
    use byteunit::ByteUnit;
    use serde::ser::SerializeStruct;
    use std::{num::ParseFloatError, path::Path};

    #[test]
    fn deser_ron_cfg() -> anyhow::Result<()> {
        set_static_workdir();
        let docker_dir = Path::new("2.0_hamm_x86_1998-07-21/docker");
        if !docker_dir.exists() {
            return Ok(());
        }

        dbg!(deser_ron::<String>(&docker_dir.join("tag.ron"))?);
        dbg!(deser_ron::<ZstdOp<PathBuf>>(&docker_dir.join("zstd.ron"))?);

        Ok(())
    }

    #[test]
    fn get_file_size() -> anyhow::Result<()> {
        let file = Path::new("tmp/zstd/2.0_hamm_x86_1998-07-21.tar.zst");
        if !file.exists() {
            return Ok(());
        }

        let meta = file.metadata()?;
        let ron = ron::to_string(&ByteUnit::new_mib(meta.len()))?;
        println!("{ron}");

        let de_ron = ron::de::from_str::<ByteUnit>(&ron)?;
        dbg!(de_ron);

        let sys_modtime = meta.modified()?;
        let time = time::OffsetDateTime::from(sys_modtime);
        dbg!(time);

        Ok(())
    }

    #[test]
    fn file_url() {
        let url = Url::parse("file:///").expect("Invalid URL");
        dbg!(url);
    }
}
