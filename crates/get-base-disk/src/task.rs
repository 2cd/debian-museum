use crate::{
    cfg::DiskV1,
    command::run_curl,
    docker::{self, spawn_docker_build},
    url::concat_url_path,
};
use log::{debug, error, info, trace};
use repack::compression::{Operation, Upack};
use std::{
    env, fs, io, iter,
    ops::Deref,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

pub(crate) fn parse_cfg(disk_cfg: &DiskV1, workdir: &Path) -> anyhow::Result<()> {
    let docker_content = fs::read_to_string(docker::DOCKER_FILE)?;

    let mirror = disk_cfg.find_mirror_url();
    let mut url_path = String::with_capacity(64);

    let mut base_name = String::with_capacity(60);
    // base + .tar
    let mut tar_fname = String::with_capacity(64);
    // base + .tar + .zstd/.zst
    let mut zstd_fname = String::with_capacity(69);
    let mut gz_fname = zstd_fname.clone();

    let mut children = Vec::with_capacity(32);
    let mut thread_pool = Vec::with_capacity(32);

    let tmp_dir = workdir.join("tmp");
    let zstd_dir = tmp_dir.join("zstd");
    fs::create_dir_all(&zstd_dir)?;
    env::set_var("DOCKER_BUILDKIT", "1");

    debug!("set current dir to {:?}", &tmp_dir);
    env::set_current_dir(&tmp_dir)?;

    let mut tag_map = docker::RepoMap::default();

    for os in disk_cfg
        .get_os()
        .iter()
        // .rev()
        .take(2)
    {
        for disk in os.get_disk() {
            concat_url_path(&mut url_path, os, disk);
            let url = mirror.join(&url_path)?;

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
            update_fname(&mut tar_fname, &base_name, &mut zstd_fname);

            // tmp + base => docker_dir
            let docker_dir = tmp_dir.join(&base_name);
            debug!("docker_dir: {:?}", docker_dir);

            // docker_dir + tar_fname => tar_path
            let tar_path =
                create_docker_file(&docker_dir, &tar_fname, &docker_content)?;

            // curl + gzip
            {
                gz_fname.clear();
                gz_fname = zstd_fname.replace(".zst", ".gz");
                debug!("running curl ...");
                run_curl(&url, &gz_fname);
                decompress_gzip(&gz_fname, &tar_path)?;
            }

            let codename_lowercase = os
                .get_codename()
                .to_ascii_lowercase();

            // get repo tags + run docker task
            {
                let repo = docker::Repository::builder()
                    .codename(&codename_lowercase)
                    .version(os.get_version())
                    .arch(disk.get_arch())
                    .tag(disk.get_tag().as_deref())
                    .project("debian")
                    .build();
                run_docker_task(repo, &mut children, &docker_dir, &mut tag_map);
            }

            // compress to zstd
            {
                let zstd_target = zstd_dir.join(&zstd_fname);
                spawn_zstd_thread(tar_path, zstd_target, &mut thread_pool);
            }
        }
    }

    wait_process(children);
    for (k, v) in tag_map.deref() {
        dbg!(k, v);
    }
    wait_threads(thread_pool)?;

    Ok(())
}

fn update_fname(tar_fname: &mut String, base_name: &str, zstd_fname: &mut String) {
    tar_fname.clear();
    tar_fname.extend([base_name, ".tar"]);

    let tar_fname = format!("{}.tar", base_name);
    zstd_fname.clear();
    zstd_fname.extend([&tar_fname, ".zst"]);
    trace!("zstd_fname capacity: {}", zstd_fname.capacity());
    debug!("tar:{}\nzstd: {}", tar_fname, zstd_fname);
}

fn run_docker_task(
    repo: docker::Repository<'_>,
    children: &mut Vec<(&str, std::process::Child)>,
    docker_dir: &Path,
    tag_map: &mut docker::RepoMap,
) {
    let ghcr_tags = repo.ghcr_repos();
    let reg_tags = repo.reg_repos();

    let tags_iter = reg_tags
        .iter()
        .map(Deref::deref)
        .chain(
            ghcr_tags
                .iter()
                .map(Deref::deref),
        );
    children.push((
        "docker-build-task",
        spawn_docker_build(
            tags_iter,
            docker::get_oci_platform(repo.arch),
            docker_dir,
        ),
    ));
    // children
    //     .push(("sleep 0.009s", command::spawn_cmd("sleep", &["0.009"])));

    let reg_iter = iter::zip(repo.reg_main_repos(), reg_tags);
    let ghcr_iter = iter::zip(repo.ghcr_main_repos(), ghcr_tags);

    // Map {key: Reg(manifest-repo-0), value: TinyVec[x86-tag0, m68k-tag0, element0...]}
    // Map {key: Reg(manifest-repo-1), value: TinyVec[x86-tag1, m68k-tag1, element1...]}
    for (key, element) in reg_iter.chain(ghcr_iter) {
        tag_map.push_to_value(key, element)
    }
}

fn create_docker_file(
    docker_dir: &Path,
    tar_fname: &str,
    docker_content: &str,
) -> io::Result<PathBuf> {
    debug!("creating the docker dir");
    fs::create_dir_all(docker_dir)?;

    let tar_path = docker_dir.join(tar_fname);
    let docker_file = docker_dir.join("Dockerfile");
    debug!("docker_file: {:?}", docker_file);

    debug!("creating a new Dockerfile");
    fs::write(docker_file, docker_content.replace("base.tar", tar_fname))?;

    Ok(tar_path)
}

fn decompress_gzip(gz_fname: &str, tar_path: &Path) -> io::Result<()> {
    let unpack_gz = Upack::new(gz_fname, tar_path);
    info!(
        "Decompressing {:?} to {:?}",
        unpack_gz.source.path, unpack_gz.target.path,
    );
    debug!("operation: {:?}", unpack_gz.operation);
    unpack_gz.run()?;
    Ok(())
}

fn spawn_zstd_thread(
    tar_path: PathBuf,
    zstd_target: PathBuf,
    thread_pool: &mut Vec<(&str, JoinHandle<io::Result<()>>)>,
) {
    let zstd_thread = thread::spawn(move || -> io::Result<()> {
        let zstd = Upack::new(&tar_path, &zstd_target)
            .with_operation(Operation::encode(Some(1)));
        info!(
            "Compressing {:?} to {:?}",
            zstd.source.path, zstd.target.path,
        );
        debug!("operation: {:?}", zstd.operation);
        zstd.run()
    });

    thread_pool.push(("compress to zstd", zstd_thread));
}

fn wait_process(children: Vec<(&str, std::process::Child)>) {
    for (name, mut task) in children {
        if task
            .try_wait()
            .is_ok_and(|x| x.is_some())
        {
            continue;
        }

        info!("wait task: {}, id: {}", name, task.id());

        if let Err(e) = task.wait() {
            error!("Task: {name}, Err: {e}");
        }
    }
}

fn wait_threads(
    thread_pool: Vec<(&str, JoinHandle<io::Result<()>>)>,
) -> anyhow::Result<()> {
    for (name, task) in thread_pool {
        if task.is_finished() {
            continue;
        }
        info!("wait thread: {}, {:?}", name, task.thread());
        task.join()
            .unwrap_or_else(|_| panic!("Failed to run task: {}", name))?
    }
    Ok(())
}
