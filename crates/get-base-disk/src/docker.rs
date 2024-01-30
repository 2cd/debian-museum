use crate::command::spawn_cmd;
use log::info;
use log_l10n::level::color::OwoColorize;
use std::process::Child;

// pub(crate) struct Docker {
//     pub(crate) tags: Vec<String>,
//     pub(crate) platform: OciPlatform,
// }
// pub enum OciPlatform {
//     LinuxAmd64,
//     LinuxArm64,
//     // Windows
// }

/// docker build -t $tag .
pub(crate) fn spawn_docker_build(tags: &[&str], platform: &str) -> Child {
    info!(
        "{docker} build {t} {tag} .",
        docker = "docker".green(),
        t = "--tag".cyan(),
        tag = tags[0].yellow()
    );

    let mut args = Vec::with_capacity(16);
    args.push("build");

    for tag in tags {
        args.extend(["--tag", tag])
    }

    args.extend(["--platform", platform, "--pull", "."]);

    spawn_cmd("docker", &args)
}
