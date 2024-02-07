use core::fmt::Display;
use log::{error, info};
use log_l10n::level::color::OwoColorize;
use std::{
    ffi::OsStr,
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{exit, Child, Command, ExitStatus, Stdio},
    sync::OnceLock,
};
use url::Url;

pub(crate) fn run_curl(url: &Url, fname: &str) {
    info!(
        "{curl} {lo} {file} {url}",
        curl = "curl".green(),
        lo = "-Lo".cyan(),
        file = fname.magenta(),
        url = url.yellow()
    );
    run("curl", &["-L", "-o", fname, url.as_str()]);
}

pub(crate) fn spawn_cmd(cmd: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .args(args)
        .spawn()
        .expect("Failed tp spawn command")
}

/// Blocks running process and does not catch stdout & stderr (i.e., defaults to direct output to the console)
pub(crate) fn run<A, S>(cmd: S, args: &[A])
where
    A: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    let status = || {
        Command::new(cmd.as_ref())
            .args(args)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("{e}");
                ExitStatus::from_raw(1)
            })
    };

    if !status().success() {
        error!(
            "Failed to run : {:?} ({:?} ...)",
            cmd.as_ref(),
            args.first().map(|x| x.as_ref())
        );
        eprintln!("Retrying ...");
        exit_if_failure(status())
    }
}

fn exit_if_failure(status: ExitStatus) {
    if !status.success() {
        exit(status.into_raw())
    }
}

/// If the current uid is not 0 (non-root user), sudo and doas are automatically detected and a new process are run synchronously and blockingly.
pub(crate) fn run_as_root<S, A>(cmd: S, args: &[A])
where
    // I: IntoIterator<Item = S>,
    A: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    // pub(crate) fn run_as_root(cmd: &str, args: &[&str]) {
    let uid = unsafe { libc::getuid() };
    log::debug!("uid: {uid}");
    if uid == 0 {
        return run(cmd, args);
    }

    let root_cmd = static_root_cmd();
    log::debug!("root_cmd: {root_cmd}");

    let mut new_args = Vec::with_capacity(args.len() + 1);
    new_args.push(cmd.as_ref());
    for a in args {
        new_args.push(a.as_ref())
    }

    info!("cmd: {root_cmd}, args: {new_args:?}");
    run(OsStr::new(root_cmd.as_ref()), &new_args);
}

#[derive(Debug, Clone, Copy)]
enum RootCmd {
    Sudo,
    Doas,
    Unknown,
}

impl Display for RootCmd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for RootCmd {
    fn as_ref(&self) -> &str {
        match self {
            Self::Doas => "doas",
            Self::Sudo => "sudo",
            _ => "please-install-sudo-or-doas-first",
        }
    }
}

fn static_root_cmd() -> &'static RootCmd {
    static D: OnceLock<RootCmd> = OnceLock::new();
    D.get_or_init(doas_or_sudo)
}

fn doas_or_sudo() -> RootCmd {
    match (cmd_exists("doas"), cmd_exists("sudo")) {
        (true, _) => RootCmd::Doas,
        (_, true) => RootCmd::Sudo,
        _ => RootCmd::Unknown,
    }
}

fn cmd_exists(bin_name: &str) -> bool {
    let which_cmd = Command::new("which")
        .arg(bin_name)
        .stdout(Stdio::null())
        .status()
        .map_or(false, |x| x.success());

    if which_cmd {
        return true;
    }

    ["/usr/bin", "/usr/local/bin", "/bin", "/sbin"]
        .into_iter()
        .any(|x| {
            log::debug!("checking for existence of file ({x}/{bin_name})");
            Path::new(x)
                .join(bin_name)
                .exists()
        })
}
