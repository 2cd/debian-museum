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
    run("curl", &["-L", "-o", fname, url.as_str()], true);
}

pub(crate) fn spawn_cmd(cmd: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .args(args)
        .spawn()
        .expect("Failed tp spawn command")
}

/// Blocks running process and does not catch stdout & stderr (i.e., defaults to direct output to the console)
pub(crate) fn run<A, S>(cmd: S, args: &[A], exit_if_failure: bool) -> ExitStatus
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

    // 1st run:
    let status_1st = status();
    if status_1st.success() {
        return status_1st;
    }

    error!(
        "Failed to run : {:?} ({:?} ...)",
        cmd.as_ref(),
        args.first().map(|x| x.as_ref())
    );
    eprintln!("Retrying ...");
    // 2nd run:
    let status_2nd = status();
    if status_1st.success() {
        return status_2nd;
    }

    eprintln!("Retrying ...");
    // 3rd run:
    let status_3rd = status();
    if exit_if_failure {
        check_status_and_exit(status_3rd)
    }
    status_3rd
}

fn check_status_and_exit(status: ExitStatus) {
    if !status.success() {
        exit(status.into_raw())
    }
}

/// If the current uid is not 0 (non-root user), sudo and doas are automatically detected and a new process are run synchronously and blockingly.
pub(crate) fn run_as_root<S, A>(
    cmd: S,
    args: &[A],
    exit_if_failure: bool,
) -> ExitStatus
where
    // I: IntoIterator<Item = S>,
    A: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    let uid = unsafe { libc::getuid() };
    log::debug!("uid: {uid}");
    if uid == 0 {
        return run(cmd, args, exit_if_failure);
    }

    let root_cmd = static_root_cmd();
    log::debug!("root_cmd: {root_cmd}");

    let mut new_args = Vec::with_capacity(args.len() + 1);

    new_args.push(cmd.as_ref());
    for a in args {
        new_args.push(a.as_ref())
    }

    info!("cmd: {root_cmd}, args: {new_args:?}");
    run(OsStr::new(root_cmd.as_ref()), &new_args, exit_if_failure)
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

/// ~= sudo fs::remove_dir_all(path)
pub(crate) fn force_remove_item_as_root<P: AsRef<Path>>(path: P) {
    #[allow(unused_variables)]
    let osstr = OsStr::new;

    let p = path.as_ref();
    // At least two levels of directories are required to avoid deleting the root directory.
    if p.components().count() <= 1 {
        return log::debug!("do nothing");
    }

    // run_as_root("chmod", &[osstr("-R"), osstr("777"), p.as_ref()]);
    run_as_root("rm", &[osstr("-rf"), p.as_ref()], true);
}

/// ~= sudo fs::rename(src, dst)
pub(crate) fn move_item_as_root<S: AsRef<OsStr>, D: AsRef<OsStr>>(src: S, dst: D) {
    run_as_root("mv", &[OsStr::new("-f"), src.as_ref(), dst.as_ref()], true);
}

// pub(crate) fn copy_dir_as_root<S: AsRef<OsStr>, D: AsRef<OsStr>>(src: S, dst: D) {
//     run_as_root("cp", &[OsStr::new("-rf"), src.as_ref(), dst.as_ref()]);
// }

pub(crate) fn create_dir_all_as_root<D: AsRef<OsStr>>(dst: D) {
    run_as_root("mkdir", &[OsStr::new("-p"), dst.as_ref()], true);
}

pub(crate) fn run_nspawn<S: AsRef<OsStr>, R: AsRef<OsStr>>(
    rootfs_dir: R,
    sh_cmd: S,
    exit_if_failure: bool,
) -> ExitStatus {
    #[allow(unused_variables)]
    let osstr = OsStr::new;

    run_as_root(
        "systemd-nspawn",
        &[
            osstr("-D"),
            rootfs_dir.as_ref(),
            osstr("-E"),
            osstr(crate::task::build_rootfs::DEB_ENV),
            // osstr("-E"),
            // osstr("LANG=en_US.UTF-8"),
            osstr("sh"),
            osstr("-c"),
            sh_cmd.as_ref(),
        ],
        exit_if_failure,
    )
}

#[cfg(test)]
mod tests {
    use crate::command::RootCmd;

    #[test]
    fn display_root_cmd() {
        let root = RootCmd::Sudo;
        assert_eq!(root.to_string(), "sudo");
    }
}
