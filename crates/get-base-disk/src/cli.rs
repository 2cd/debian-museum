use clap::{value_parser, Parser};
use getset::Getters;
use log::trace;
use std::{path::PathBuf, process::exit};

use crate::{
    cfg::disk::DiskV1,
    docker::repo::Repository,
    task::{old_old_debian, pool::global_pool},
    url::concat_url_path,
};

pub(crate) const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Getters)]
#[getset(get = "pub(crate) with_prefix")]
#[command(arg_required_else_help = true)]
/// Example: --os debian --ver 2.2 --tag base --obtain --build
pub(crate) struct Cli {
    /// OS Name, e.g. debian, ubuntu
    #[arg(long, id = "OS_Name")]
    os: String,

    /// Version, e.g. 1.3, 2.0, 22.04
    #[arg(long)]
    ver: String,

    /// e.g. base
    #[arg(long)]
    tag: Option<String>,

    /// download or build rootfs
    #[arg(long, help_heading = "Operation")]
    obtain: bool,

    /// pack to tar & compress to zstd
    #[arg(long, help_heading = "Operation")]
    repack: bool,

    /// zstd compression level (0 ~ 22)
    #[arg(long, help_heading = "Operation", value_parser = value_parser!(u8).range(0..=22), requires = "repack")]
    zstd_level: Option<u8>,

    /// build container
    #[arg(long, help_heading = "Docker")]
    build: bool,

    /// push to ghcr & reg
    #[arg(long, help_heading = "Docker")]
    push: bool,

    /// i.e. docker:x86 + docker:arm -> docker:latest
    #[arg(long, help_heading = "Docker")]
    create_manifest: bool,

    /// repo-digest xx/yy@sha256:123456abcdef
    #[arg(long, help_heading = "Docker")]
    update_repo_digest: bool,

    /// generate digests (yaml or ron)
    #[arg(
        long,
        group = "digests",
        id = "/path/to/file.yml",
        help_heading = "Save-Config"
    )]
    digest: Option<PathBuf>,

    #[arg(long, help = PKG_VERSION, help_heading = "Builtin")]
    version: bool,
}

impl Cli {
    fn is_old_old_debian(&self) -> bool {
        if self.get_os() == "debian"
            && old_old_debian::VERS.contains(&self.get_ver().as_str())
        {
            if self.get_ver() == "2.2" {
                return matches!(self.get_tag().as_deref(), Some("base"));
            }
            return true;
        }
        false
    }
    pub(crate) fn run(&self) -> anyhow::Result<()> {
        self.print_version();

        if self.is_old_old_debian() {
            self.parse_old_old_debian()?;
            return Ok(());
        }

        log::debug!("Not old old debian");
        Ok(())
    }

    fn parse_old_old_debian(&self) -> anyhow::Result<()> {
        let cfg = DiskV1::deser()?;
        let mut repos = tinyvec::TinyVec::<[Repository; 16]>::new();
        {
            let mirror = cfg.find_mirror_url();
            let mut url_path = String::with_capacity(64);

            for os in cfg
                .get_os()
                .iter()
                .filter(|o| o.get_version() == self.get_ver())
            {
                for disk in os.get_disk() {
                    concat_url_path(&mut url_path, os, disk);

                    let repo = Repository::builder()
                        .codename(os.get_codename())
                        .arch(disk.get_arch())
                        .tag(disk.get_tag().as_deref())
                        .version(os.get_version())
                        .project("debian")
                        .url(mirror.join(&url_path)?)
                        .date(disk.get_date())
                        .build();
                    repos.push(repo)
                }
            }
        }

        if *self.get_obtain() {
            old_old_debian::obtain(&repos)?;
        }

        if *self.get_repack() {
            old_old_debian::repack(&repos, self.get_zstd_level().as_ref())?;
        }

        if *self.get_build() {
            old_old_debian::docker_task::docker_build(&repos)?;
        }

        if *self.get_push() {
            old_old_debian::docker_task::docker_push(&repos)?;
        }

        if *self.get_create_manifest() {
            old_old_debian::docker_task::create_manifest(&repos)?;
        }

        if *self.get_update_repo_digest() {
            old_old_debian::docker_task::pull_image_and_create_repo_digests(&repos)?;
        }

        if let Some(p) = self.get_digest() {
            old_old_debian::digest_cfg::create_digest_cfg(&repos, p)?;
        }

        global_pool().join();

        Ok(())
    }

    fn print_version(&self) {
        if !*self.get_version() {
            return;
        }

        trace!("print version info:");
        println!(
            "version: {}\n\
                name: {}\n\
            ",
            // git-hash: git rev-parse HEAD
            //
            PKG_VERSION,
            log_l10n::get_pkg_name!(),
        );
        exit(0)
    }
}
