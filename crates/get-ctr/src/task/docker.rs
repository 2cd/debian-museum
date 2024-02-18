use log_l10n::level::color::OwoColorize;

use crate::{
    command::{self, run},
    docker::{self, repo::Repository, spawn_docker_build},
    task::compression::{extract_tar_as_root, pack_tar_as_root},
};
use std::{
    self,
    env::{self, temp_dir},
    ffi::OsStr,
    fs, io, iter,
    ops::Deref,
    path::Path,
};
pub(crate) fn run_docker_push(repo: &str) {
    log::info!(
        "{} {} {} {}",
        "docker".green(),
        "push".yellow(),
        "--all-tags".cyan(),
        repo.blue()
    );
    command::run("docker", &["push", "--all-tags", repo], true);
}

pub(crate) fn run_docker_build(
    repo: &docker::repo::Repository<'_>,
    children: &mut Vec<(&str, std::process::Child)>,
    docker_dir: &Path,
    tag_map: &mut docker::repo_map::RepoMap,
) -> anyhow::Result<()> {
    let (ghcr_tags, reg_tags) = match repo.get_date_tagged() {
        true => (repo.ghcr_date_tagged_repos(), repo.reg_date_tagged_repos()),
        _ => (repo.ghcr_repos(), repo.reg_repos()),
    };

    // ghcr_tags => docker-dir/ghcr.ron
    // tag => docker-dir/tag.ron
    {
        fs::write(docker_dir.join("ghcr.ron"), ron::to_string(&ghcr_tags)?)?;
        fs::write(docker_dir.join("reg.ron"), ron::to_string(&reg_tags)?)?;

        let tag = ghcr_tags[0]
            .rsplit(':')
            .next()
            .unwrap_or("latest");
        fs::write(docker_dir.join("tag.ron"), ron::to_string(tag)?)?;
    }

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
            docker::get_oci_platform(repo.get_arch()),
            docker_dir,
        ),
    ));
    // children
    //     .push(("sleep 0.009s", command::spawn_cmd("sleep", &["0.009"])));

    let (ghcr_main, reg_main) = match repo.get_date_tagged() {
        true => (
            repo.ghcr_main_date_tagged_repos(),
            repo.reg_main_date_tagged_repos(),
        ),
        _ => (repo.ghcr_main_repos(), repo.reg_main_repos()),
    };

    let ghcr_iter = iter::zip(ghcr_main, ghcr_tags);
    let reg_iter = iter::zip(reg_main, reg_tags);

    // Map {key: Reg(manifest-repo-0), value: TinyVec[x86-tag0, m68k-tag0, element0...]}
    // Map {key: Reg(manifest-repo-1), value: TinyVec[x86-tag1, m68k-tag1, element1...]}
    for (key, element) in reg_iter.chain(ghcr_iter) {
        tag_map.push_to_value(key, element)
    }
    Ok(())
}

pub(crate) fn save_cache(first_repo: &Repository<'_>) -> io::Result<()> {
    let base_name = first_repo.base_name();
    let tmp_dir = temp_dir().join(&base_name);
    let tar_path = tmp_dir.join("cache.tar");

    pack_tar_as_root(".", &tar_path, false);

    const CONTENT: &str = r##"# syntax=docker/dockerfile:1
FROM busybox:musl
COPY cache.tar /
"##;

    let docker_file = tmp_dir.join("Dockerfile");

    fs::write(docker_file, CONTENT)?;

    let osstr = OsStr::new;

    let docker_tag = get_cache_tag(first_repo, &base_name);

    let build_args = [
        osstr("build"),
        osstr("--tag"),
        osstr(&docker_tag),
        tmp_dir.as_ref(),
    ];
    log::info!("cmd: docker, args: {build_args:?}");
    run("docker", &build_args, true);

    let push_args = ["push", &docker_tag].map(osstr);
    log::info!("cmd: docker, args: {push_args:?}");
    run("docker", &push_args, true);

    Ok(())
}

fn get_cache_tag(first_repo: &Repository<'_>, base_name: &str) -> String {
    format!(
        "{uri}/{owner}/cache:{base_name}",
        uri = Repository::REG_URI,
        owner = first_repo.get_reg_date_tagged_owner(),
    )
}

pub(crate) fn restore_cache(first_repo: &Repository<'_>) -> io::Result<()> {
    let base_name = first_repo.base_name();
    let docker_tag = get_cache_tag(first_repo, &base_name);
    let args = [
        "run",
        "-t",
        "--rm",
        "-v",
        &format!("{}:/host", env::current_dir()?.to_string_lossy()),
        "--pull",
        "always",
        &docker_tag,
        "mv",
        "/cache.tar",
        "/host",
    ];
    log::info!("cmd: docker, args: {args:?}");
    run("docker", &args, true);

    extract_tar_as_root(Path::new("cache.tar"), ".")?;
    Ok(())
}
