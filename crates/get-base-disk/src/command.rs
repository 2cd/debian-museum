use log::{error, info};
use log_l10n::level::color::OwoColorize;
use std::{
    os::unix::process::ExitStatusExt,
    process::{exit, Child, Command, ExitStatus},
};

pub(crate) fn run_curl(url: &str, fname: &str) {
    info!(
        "{curl} {lo} {file} {url}",
        curl = "curl".green(),
        lo = "-Lo".cyan(),
        file = fname.magenta(),
        url = url.yellow()
    );
    run("curl", &["-L", "-o", fname, url]);
}

pub(crate) fn spawn_cmd(cmd: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .args(args)
        .spawn()
        .expect("Failed tp spawn command")
}

pub(crate) fn run(cmd: &str, args: &[&str]) {
    let status = || {
        Command::new(cmd)
            .args(args)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("{e}");
                ExitStatus::from_raw(1)
            })
    };

    if !status().success() {
        error!("Failed to run : {} {:?}", cmd, args);
        eprintln!("Retrying ...");
        exit_if_failure(status())
    }
}

fn exit_if_failure(status: ExitStatus) {
    if !status.success() {
        exit(status.into_raw())
    }
}
