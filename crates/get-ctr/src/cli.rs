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
    #[arg(long, id = "OS_Name", default_value = "debian")]
    os: String,

    /// Version, e.g. 1.3, 2.0, 22.04
    #[arg(long, default_value = "2.1")]
    ver: String,

    /// e.g. base
    #[arg(long, num_args = 0..=1, default_missing_value = " ")]
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

    /// generate digests(e.g., --digest a.yml --digest a.ron)
    #[arg(
        long,
        group = "digests",
        id = "Vec</path/to/file>",
        help_heading = "Save Config",
        num_args = 0..=1,
        default_missing_value = " ",
    )]
    digest: Option<Vec<PathBuf>>,

    /// generate title content for releases
    #[arg(long, help_heading = "Save Config")]
    title: bool,

    #[arg(long, help_heading = "Save Config")]
    release_tag: bool,

    // #[arg(
    //     long,
    //     help_heading = "Save Config",
    //     num_args = 0..=1,
    //     default_missing_value = " ",
    // )]
    // title: Option<PathBuf>,
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
                        .tag(match disk.get_tag() {
                            Some(x) if x.trim().is_empty() => None,
                            x => x.as_deref(),
                        })
                        .version(os.get_version())
                        .project("debian")
                        .url(mirror.join(&url_path)?)
                        .date(disk.get_date())
                        .title_date(os.get_date())
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

        let first = || {
            repos
                .iter()
                .next()
                .expect("Empty Repos")
        };

        if *self.get_title() {
            print_title(first());
        }

        if *self.get_release_tag() {
            let first = first();
            println!("{}{}", first.version, first.opt_tag_suffix());
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

fn print_title(first: &Repository<'_>) {
    let (date_prefix, date, date_suffix) = match first.title_date {
        Some(d) => (" (", d, ")"),
        _ => ("", "", ""),
    };
    let (tag, tag_suffix) = match first.tag {
        Some(p) => (p, ", "),
        _ => ("", ""),
    };
    println!(
        "{} {}{}{}{}{}{}",
        first.version,
        first.codename,
        date_prefix,
        tag,
        tag_suffix,
        date,
        date_suffix
    );
}
