use crate::{
    cfg::{
        debootstrap::{self, Source},
        disk::DiskV1,
    },
    docker::repo::{Repository, SrcFormat},
    logger,
    task::{
        build_rootfs,
        docker::{restore_cache, save_cache},
        old_old_debian,
        pool::global_pool,
    },
    url::concat_url_path,
};
use anyhow::{bail, Context};
use clap::{value_parser, Parser};
use getset::Getters;
use log::trace;
use std::{path::PathBuf, process::exit};

pub(crate) const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Getters)]
#[getset(get = "pub(crate) with_prefix")]
#[command(arg_required_else_help = true)]
/// Example: --os debian --ver 2.2 --tag base --obtain --build
pub(crate) struct Cli {
    /// OS Name, e.g., debian, ubuntu
    #[arg(long, id = "OS_Name", default_value = "debian")]
    os: String,

    /// Version, e.g., 1.3, 2.0, 22.04
    #[arg(long, default_value = "2.1")]
    ver: String,

    /// If no architecture is specified, all is used by default.
    /// Specifying debian 1.3, 2.0, 2.1, 2.2-base architecture is not supported.
    #[arg(long, num_args = 0..=1, default_missing_value = " ")]
    arch: Option<String>,

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

    /// i.e., docker:x86 + docker:arm -> docker:latest
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

    /// pack the [workdir] to cache.tar, then build & push to REG  
    #[arg(long, help_heading = "CI", group = "cache")]
    save_cache: bool,

    #[arg(long, help_heading = "CI", group = "cache")]
    restore_cache: bool,

    #[arg(long, help = PKG_VERSION, help_heading = "Info")]
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

        crate::dir::set_static_workdir();

        if self.is_old_old_debian() {
            self.handle_old_old_debian()?;
            return Ok(());
        }

        log::debug!("Not old old debian");

        match self.get_os().as_ref() {
            "debian" => self.handle_debian()?,
            // "ubuntu"
            _ => self.handle_ubuntu()?,
        }
        Ok(())
    }

    fn handle_old_old_debian(&self) -> anyhow::Result<()> {
        let cfg = DiskV1::deser()?;
        let mut repos = tinyvec::TinyVec::<[Repository; 16]>::new();
        {
            let mirror = crate::url::debian_archive()?;
            let mut url_path = String::with_capacity(64);

            for os in cfg
                // .get_os()
                .iter()
                .filter(|o| o.get_version() == self.get_ver())
            {
                for disk in os.get_disk() {
                    concat_url_path(&mut url_path, os, disk);
                    let codename_lower = os
                        .get_codename()
                        .to_ascii_lowercase();

                    let repo = Repository::builder()
                        .codename(os.get_codename())
                        .series(codename_lower)
                        .arch(disk.get_arch())
                        .tag(match disk.get_tag() {
                            Some(x) if x.trim().is_empty() => None,
                            x => x.as_deref(),
                        })
                        .version(os.get_version())
                        .project("debian")
                        .osname("Debian")
                        .url(mirror.join(&url_path)?)
                        .date(disk.get_date())
                        .title_date(os.get_date())
                        .patch(os.get_patch())
                        .build();
                    repos.push(repo)
                }
            }
        }

        if *self.get_obtain() {
            old_old_debian::obtain(&repos)?;
        }

        self.cli_common(repos.to_vec())?;

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

    /// debian 2.2 ~ sid
    fn handle_debian(&self) -> anyhow::Result<()> {
        let (project, date_tagged) = match self.get_ver().as_str() {
            "sid" | "unstable" => ("debian-sid", true),
            _ => ("debian", false),
        };
        self.handle_modern_os(project, date_tagged)
    }

    /// handles all ubuntu versions
    fn handle_ubuntu(&self) -> anyhow::Result<()> {
        let (project, date_tagged) = match self.get_ver().as_str() {
            "devel" | "unstable" => ("ubuntu-dev", true),
            _ => ("ubuntu", false),
        };
        self.handle_modern_os(project, date_tagged)
    }

    fn handle_modern_os(
        &self,
        project: &str,
        date_tagged: bool,
    ) -> anyhow::Result<()> {
        log::debug!("parsing the {project} (ron config)");
        let ron_str = match project {
            "debian" | "debian-sid" => debootstrap::DEBIAN_RON,
            _ => debootstrap::UBUNTU_RON,
        };

        let cfg = ron::from_str::<debootstrap::Cfg>(ron_str)
            .context("Failed to parse ron")?;
        log::trace!("cfg: {cfg:?}");

        let mut repos = tinyvec::TinyVec::<[Repository; 20]>::new();

        for os in cfg.iter().filter(|o| {
            o.get_version()
                .split_ascii_whitespace()
                .next()
                == Some(self.get_ver())
        }) {
            let suite = os.get_series();
            let main_src = os.get_source();
            let main_deb_src = main_src.debootstrap_src(suite);

            for tag in os
                .get_tag()
                .iter()
                .filter(|sub| match self.get_arch().as_deref() {
                    Some(arch) if arch.trim().is_empty() => true,
                    Some(arch) => [sub.get_arch(), sub.get_deb_arch()]
                        .map(|x| x.as_str())
                        .contains(&arch),
                    _ => true,
                })
            {
                let sub_src = tag.get_source();
                let src_fmt = get_src_format(sub_src, main_src);

                let sub_deb_src = sub_src.debootstrap_src(suite);

                let deb_src = match (sub_deb_src, main_deb_src.clone()) {
                    (Some(s), _) => s,
                    (_, Some(s)) => s,
                    _ => bail!("Empty Debootstrap Source"),
                };

                let repo_version = match project {
                    "debian-sid" | "ubuntu-dev" => "unstable",
                    // "ubuntu-dev" => "devel",
                    _ => os
                        .get_version()
                        .split_ascii_whitespace()
                        .next()
                        .expect("Invalid Version"),
                };
                let title_date = match os.get_date() {
                    s if !s.is_empty() => s,
                    _ => logger::today(),
                };

                let repo = Repository::builder()
                    .osname(os.get_name())
                    .arch(tag.get_arch())
                    .codename(os.get_codename())
                    .series(os.get_series())
                    .title_date(title_date)
                    .version(repo_version)
                    .no_minbase(*os.get_no_minbase())
                    .deb822(*os.get_deb822_format())
                    .debootstrap_src(deb_src)
                    .deb_arch(tag.get_deb_arch())
                    .project(project)
                    .source(src_fmt)
                    .components(os.get_components().as_deref())
                    .date_tagged(date_tagged)
                    .build();
                repos.push(repo)
            }
        }
        repos.reverse();

        if *self.get_obtain() {
            build_rootfs::obtain(&repos)?;
        }

        self.cli_common(repos.to_vec())?;
        Ok(())
    }

    fn cli_common(&self, repos: Vec<Repository>) -> Result<(), anyhow::Error> {
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
                .first()
                .expect("Empty Repos")
        };
        if *self.get_title() {
            print_title(first());
        }
        if *self.get_release_tag() {
            let first = first();
            println!("{}{}", first.get_version(), first.opt_tag_suffix());
        };
        global_pool().join();

        if *self.get_restore_cache() {
            restore_cache(first())?;
            exit(0)
        }

        if *self.get_save_cache() {
            save_cache(first())?;
            exit(0)
        }

        Ok(())
    }
}

fn get_src_format(sub_src: &Source, main_src: &Source) -> SrcFormat {
    match (
        (sub_src.get_src(), sub_src.get_enabled()),
        (main_src.get_src(), main_src.get_enabled()),
    ) {
        ((Some(s), _), ..) => SrcFormat::Simple(s.into()),
        ((_, Some(s)), ..) => SrcFormat::Complex {
            enabled: s.to_owned(),
            disabled: sub_src.disabled_srcs_owned(),
        },
        (_, (Some(s), ..)) => SrcFormat::Simple(s.to_owned()),
        (_, (_, Some(s))) => SrcFormat::Complex {
            enabled: s.to_owned(),
            disabled: main_src.disabled_srcs_owned(),
        },
        _ => panic!("Invalid Sources"),
    }
}

// pub(crate) type Systems = TinyVec<[debootstrap::OS; 20]>;

fn print_title(first: &Repository<'_>) {
    let (date_prefix, date, date_suffix) = match first.get_title_date() {
        Some(d) => (" (", *d, ")"),
        _ => ("", "", ""),
    };

    let (tag, tag_suffix) = match (first.get_tag(), date) {
        (Some(p), d) if !d.is_empty() => (*p, ", "),
        (Some(p), _) => (*p, ""),
        _ => ("", ""),
    };

    if *first.get_date_tagged() {
        return println!(
            "{name} ({tag}{tag_suffix}{date})",
            name = first.get_codename()
        );
    }

    println!(
        "{} {}{}{}{}{}{}",
        first.get_version(),
        first.get_codename(),
        date_prefix,
        tag,
        tag_suffix,
        date,
        date_suffix
    );
}
