use log_l10n::level::color::OwoColorize;

use crate::{
    command,
    docker::{self, spawn_docker_build},
};
use std::{self, fs, iter, ops::Deref, path::Path};
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
    let ghcr_tags = repo.ghcr_repos();
    let reg_tags = repo.reg_repos();

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

    let reg_iter = iter::zip(repo.reg_main_repos(), reg_tags);
    let ghcr_iter = iter::zip(repo.ghcr_main_repos(), ghcr_tags);

    // Map {key: Reg(manifest-repo-0), value: TinyVec[x86-tag0, m68k-tag0, element0...]}
    // Map {key: Reg(manifest-repo-1), value: TinyVec[x86-tag1, m68k-tag1, element1...]}
    for (key, element) in reg_iter.chain(ghcr_iter) {
        tag_map.push_to_value(key, element)
    }
    Ok(())
}
