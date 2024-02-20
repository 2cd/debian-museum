use crate::command::spawn_cmd;
use log::{debug, info};
use std::{path::Path, process::Child};

pub(crate) mod repo;
pub(crate) mod repo_map;
// pub(crate) mod
pub(crate) const DOCKER_FILE_OLD_CONTENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/old_old_debian/Dockerfile"
));

pub(crate) const DOCKER_FILE_FOR_NEW_DISTROS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/debootstrap/Dockerfile"
));

pub(crate) const DOCKER_IGNORE_CONTENT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/.dockerignore"));

pub(crate) fn get_oci_platform(arch: &str) -> &str {
    archmap::linux_oci_platform::map()
        .get(arch)
        // .copied()
        .expect("linux/amd64")
}
/// `docker build --tag $tag0 --tag $tag1 ...`
pub(crate) fn spawn_docker_build<'a, T: IntoIterator<Item = &'a str>>(
    tags: T,
    platform: &str,
    context: &Path,
) -> Child {
    debug!("building the docker container ...");
    let mut args = Vec::with_capacity(16);
    args.push("build");

    for tag in tags {
        info!("tag:\t {}", tag);
        args.extend(["--tag", tag])
    }

    let context_path_str = context.to_string_lossy();
    args.extend(["--platform", platform, "--pull", &context_path_str]);

    spawn_cmd("docker", &args)
}
