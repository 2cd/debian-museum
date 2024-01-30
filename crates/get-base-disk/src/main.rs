mod cfg;
mod command;
mod docker;
mod logger;
mod url;

use log::{debug, info, trace};
use repack::compression::{Operation, Upack};

use crate::{
    cfg::DiskV1, command::run_curl, docker::spawn_docker_build, url::concat_url_path,
};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    thread,
};

fn main() -> anyhow::Result<()> {
    logger::init();

    let workdir = set_workdir()?;
    info!("Working directory: {:?}", workdir);

    let disk_cfg = cfg::DiskV1::deser()?;
    parse_cfg(&disk_cfg, &workdir)?;

    Ok(())
}

const DOCKER_FILE: &str = "assets/ci/base/Dockerfile";

fn parse_cfg(disk_cfg: &DiskV1, workdir: &Path) -> anyhow::Result<()> {
    let docker_content = fs::read_to_string(DOCKER_FILE)?;

    let mirror = disk_cfg.find_mirror_url();
    let mut url_path = String::with_capacity(64);

    let mut base_name = String::with_capacity(60);
    // base + .tar
    let mut tar_fname = String::with_capacity(64);
    // base + .tar + .zstd/.zst
    let mut zstd_fname = String::with_capacity(69);
    let mut gz_fname = zstd_fname.clone();

    let mut children = Vec::with_capacity(32);
    let mut threads = Vec::with_capacity(32);

    let tmp_dir = workdir.join("tmp");
    let zstd_dir = tmp_dir.join("zstd");
    fs::create_dir_all(&zstd_dir)?;
    env::set_var("DOCKER_BUILDKIT", "1");

    for os in disk_cfg
        .get_os()
        .iter()
        .rev()
        .take(1)
    {
        for disk in os.get_disk() {
            concat_url_path(&mut url_path, os, disk);
            let url = mirror.join(&url_path)?;

            debug!("set current dir to {:?}", &tmp_dir);
            env::set_current_dir(&tmp_dir)?;

            base_name.clear();
            // name: "os.version _ os.codename _ disk.arch _ disk.tag _ disk.date".tar
            base_name = format!(
                "{}_{}_{}_{}_{}",
                os.get_version(),
                os.get_codename(),
                disk.get_arch(),
                disk.get_tag()
                    .as_deref()
                    .unwrap_or_default(),
                disk.get_date()
            );

            tar_fname.clear();
            tar_fname.extend([&base_name, ".tar"]);

            let tar_fname = format!("{}.tar", base_name);
            zstd_fname.clear();
            zstd_fname.extend([&tar_fname, ".zst"]);
            trace!("zstd_fname capacity: {}", zstd_fname.capacity());
            debug!("tar:{}\nzstd: {}", tar_fname, zstd_fname);

            // tmp + base + docker => docker_dir
            let docker_dir = PathBuf::from_iter([
                tmp_dir.as_path(),
                Path::new(&base_name),
                Path::new("docker"),
            ]);
            info!("docker_dir: {:?}", docker_dir);

            debug!("creating the docker dir");
            fs::create_dir_all(&docker_dir)?;
            let tar_path = docker_dir.join(&tar_fname);

            let docker_file = docker_dir.join("Dockerfile");
            debug!("docker_file: {:?}", docker_file);
            debug!("creating a new Dockerfile");
            fs::write(docker_file, docker_content.replace("base.tar", &tar_fname))?;

            gz_fname.clear();
            gz_fname = zstd_fname.replace(".zst", ".gz");

            debug!("running curl ...");
            run_curl(url.as_str(), &gz_fname);

            let unpack_gz = Upack::new(&gz_fname, &tar_path);
            info!(
                "Decompressing {:?} to {:?}",
                unpack_gz.source.path, unpack_gz.target.path,
            );
            debug!("operation: {:?}", unpack_gz.operation);
            unpack_gz.run()?;

            debug!("set current dir to {:?}", &docker_dir);
            env::set_current_dir(&docker_dir)?;

            children.push((
                "docker build",
                // tag1: 2.0-x86
                // tag2: hamm-x86
                spawn_docker_build(
                    &[&format!(
                        "tmp_{}",
                        os.get_codename()
                            .to_ascii_lowercase()
                    )],
                    // TODO: add archmap (OciPlatform table)
                    &format!("linux/{}", disk.get_arch()),
                ),
            ));
            // children.push(("sleep 2s", command::spawn_cmd("sleep", &["0.009"])));

            let zstd_target = zstd_dir.join(&zstd_fname);
            let zstd_thread = thread::spawn(move || -> io::Result<()> {
                let zstd = Upack::new(&tar_path, &zstd_target)
                    .with_operation(Operation::encode(Some(18)));
                info!(
                    "Compressing {:?} to {:?}",
                    zstd.source.path, zstd.target.path,
                );
                debug!("operation: {:?}", zstd.operation);
                zstd.run()
            });

            threads.push(("compress to zstd", zstd_thread));
        }
    }

    for (name, task) in threads {
        if task.is_finished() {
            continue;
        }
        info!("wait thread: {}, {:?}", name, task.thread());
        task.join()
            .unwrap_or_else(|_| panic!("Failed to run task: {}", name))?
    }

    for (name, mut task) in children {
        if task
            .try_wait()
            .is_ok_and(|x| x.is_some())
        {
            continue;
        }

        info!("wait task: {}, id: {}", name, task.id());
        dbg!(task.wait()?);
    }

    Ok(())
}

fn set_workdir() -> io::Result<PathBuf> {
    let pwd = env::current_dir()?;

    #[cfg(debug_assertions)]
    {
        if pwd.ends_with(Path::new("crates").join(log_l10n::get_pkg_name!())) {
            log::info!("set current dir to ../..");
            env::set_current_dir("../..")?;
            return env::current_dir();
        }
    }

    Ok(pwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::DiskV1;
    use std::fs;

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
        use ron::{extensions::Extensions, ser::PrettyConfig};

        let ron_file = DiskV1::DISK_RON;
        // logger::init();
        set_workdir()?;
        dbg!(env::current_dir());
        let toml_path = Path::new(ron_file).with_extension("toml");
        dbg!(&toml_path);

        let disk_v1 = toml::from_str::<DiskV1>(&fs::read_to_string(toml_path)?)?;

        let pretty = PrettyConfig::default()
            .enumerate_arrays(true)
            // .extensions(Extensions::IMPLICIT_SOME),
            .depth_limit(4);

        let ron_str = ron::ser::to_string_pretty(&disk_v1, pretty)?;
        fs::write(ron_file, ron_str)?;

        Ok(())
    }
}
