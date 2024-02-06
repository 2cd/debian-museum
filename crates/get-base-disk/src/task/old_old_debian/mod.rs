use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub(crate) mod digest_cfg;
pub(crate) mod docker_task;

use crate::{
    command::run_curl,
    docker::repo::Repository,
    task::compression::{decompress_gzip, spawn_zstd_thread},
};
use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
pub(crate) const VERS: [&str; 4] = ["1.3", "2.0", "2.1", "2.2"];
pub(crate) const BUILD_TIME_RON: &str = "build-time.ron";

pub(crate) struct TarFile {
    // base: String,
    pub(crate) tar_fname: String,
    pub(crate) tar_path: PathBuf,
    pub(crate) docker_dir: PathBuf,
}

impl<'r> Repository<'r> {
    fn generate_tar_path(base: &str, tar_fname: &str) -> PathBuf {
        PathBuf::from_iter([base, "docker", tar_fname])
    }

    fn docker_ron_filename(&self) -> String {
        let suffix = self.opt_tag_suffix();
        format!("{}-{}{}.ron", self.version, self.codename, suffix)
    }

    fn base_tar_name(&self) -> io::Result<TarFile> {
        let base: String = self.base_name();
        let tar_fname = format!("{base}.tar");

        let tar_path = Self::generate_tar_path(&base, &tar_fname);
        log::debug!("tar_path: {tar_path:?}");

        // create dir
        let docker_dir = tar_path_parent(&tar_path)?.to_owned();
        if !docker_dir.exists() {
            log::debug!("creating the tar_path.parent() dir");
            fs::create_dir_all(&docker_dir)?
        };

        Ok(TarFile {
            tar_fname,
            tar_path,
            docker_dir,
        })
    }
}

pub(crate) fn obtain<'a, I: IntoIterator<Item = &'a Repository<'a>>>(
    repos: I,
) -> anyhow::Result<()> {
    for r in repos {
        log::trace!("{r:#?}");

        let TarFile {
            tar_fname,
            tar_path,
            ref docker_dir,
            ..
        } = r.base_tar_name()?;

        let build_time = ron::to_string(&time::OffsetDateTime::now_utc())?;
        fs::write(docker_dir.join(BUILD_TIME_RON), build_time)?;

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
            ref docker_dir,
            ..
        } = r.base_tar_name()?;

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
            fs::write(docker_dir.join("zstd.ron"), ron::to_string(&op)?)?;
        }

        // compress to zstd
        spawn_zstd_thread(tar_path, zstd_file, zstd_lv);
    }
    Ok(())
}

pub(crate) fn tar_path_parent(tar_path: &Path) -> io::Result<&Path> {
    tar_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Invalid Tar Path"))
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

        dbg!(deser_ron::<String, _>(&docker_dir.join("tag.ron"))?);
        dbg!(deser_ron::<ZstdOp<PathBuf>, _>(
            &docker_dir.join("zstd.ron")
        )?);

        Ok(())
    }

    #[test]
    fn get_file_size() -> anyhow::Result<()> {
        let file = Path::new("tmp/zstd/2.0_hamm_x86_1998-07-21.tar.zst");
        if !file.exists() {
            return Ok(());
        }

        let meta = file.metadata()?;
        let ron = ron::to_string(&ByteUnit::new(meta.len()))?;
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
        let url = url::Url::parse("file:///").expect("Invalid URL");
        dbg!(url);
    }
}
