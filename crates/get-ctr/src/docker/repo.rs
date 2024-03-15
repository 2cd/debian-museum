use getset::Getters;
use std::{
    borrow::Cow,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};
use tinyvec::TinyVec;
use typed_builder::TypedBuilder;
use url::Url;

use crate::{
    cfg::{
        components,
        debootstrap::DebootstrapSrc,
        disk::OsPatch,
        mirror::{self, static_debian_snapshot, ubuntu, ubuntu_ports},
    },
    command::run_and_get_stdout,
    docker::{get_oci_platform, repo_map},
    logger::{self, today_date},
};

#[derive(Getters, TypedBuilder, Debug, Clone)]
#[getset(get = "pub(crate) with_prefix")]
#[builder(field_defaults(default))]
pub(crate) struct Repository<'r> {
    #[builder(default = "2cd")]
    owner: &'r str,
    #[builder(default = "debian")]
    project: &'r str,

    #[builder(default = "Debian")]
    osname: &'r str,

    #[builder(!default)]
    codename: &'r str,

    // #[builder(!default, setter(transform = |s: &str| s.to_ascii_lowercase()))]
    #[builder(!default, setter(into))]
    series: String,

    #[builder(!default)]
    version: &'r str,
    #[builder(!default)]
    arch: &'r str,

    tag: Option<&'r str>,

    /// old old debian floppy file creation date
    #[builder(default = "1900-01-01")]
    date: &'r str,

    #[builder(setter(strip_option))]
    url: Option<Url>,

    #[builder(setter(strip_option))]
    title_date: Option<&'r str>,

    #[builder(setter(into))]
    patch: Option<&'r OsPatch>,

    deb822: bool,

    no_minbase: bool,

    #[builder(setter(strip_option))]
    deb_arch: Option<&'r str>,

    components: Option<&'r str>,

    #[builder(setter(strip_option))]
    source: Option<SrcFormat>,

    #[builder(setter(strip_option))]
    debootstrap_src: Option<DebootstrapSrc>,

    date_tagged: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum SrcFormat {
    Simple(String),
    Complex {
        enabled: Vec<String>,
        disabled: Option<Vec<String>>,
    },
}

impl SrcFormat {
    pub(crate) fn create_src_list(
        &self,
        series: &str,
        mirror_dir: &Path,
        // deb822: bool,
        components: Option<&str>,
    ) -> anyhow::Result<()> {
        match self {
            /*
            deb https://mirror.sjtu.edu.cn/ubuntu/ jammy main restricted universe multiverse
            # deb-src https://mirror.sjtu.edu.cn/ubuntu/ jammy main restricted universe multiverse

            deb https://mirror.sjtu.edu.cn/ubuntu/ jammy-updates main restricted universe multiverse
            # deb-src https://mirror.sjtu.edu.cn/ubuntu/ jammy-updates main restricted universe multiverse

            deb https://mirror.sjtu.edu.cn/ubuntu/ jammy-backports main restricted universe multiverse
            # deb-src https://mirror.sjtu.edu.cn/ubuntu/ jammy-backports main restricted universe multiverse

            deb https://mirror.sjtu.edu.cn/ubuntu/ jammy-security main restricted universe multiverse
            # deb-src https://mirror.sjtu.edu.cn/ubuntu/ jammy-security main restricted universe multiverse

            # --------
            # Disabled
            # deb https://mirror.sjtu.edu.cn/ubuntu/ jammy-proposed main restricted universe multiverse
            # deb-src https://mirror.sjtu.edu.cn/ubuntu/ jammy-proposed main restricted universe multiverse
            */
            Self::Simple(s) => {
                let mirrors = match s.as_str() {
                    "ubuntu" => mirror::ubuntu::mirrors(),
                    "ubuntu-ports" => mirror::ubuntu_ports::mirrors(),
                    _ => mirror::ubuntu_old::mirrors(),
                };
                for m in mirrors {
                    let url = https_to_http(&m);
                    let name = m.get_name();
                    let one_line_style = ubuntu_one_line_style(series, &url);
                    let deb822_style = ubuntu_deb822_style(series, &url, name);
                    let legacy_file = legacy_src_list_path(&m, mirror_dir, name);
                    let deb822_file = legacy_file.with_extension("sources");
                    fs::write(legacy_file, one_line_style)?;
                    fs::write(deb822_file, deb822_style)?;
                }
            }
            Self::Complex {
                enabled,
                disabled: disabled_srcs,
            } => {
                let components = get_debian_components(components);

                let mut official_legacy_style = String::with_capacity(256);
                let mut cdn_legacy_style = String::with_capacity(256);

                let mut official_deb822_style = String::with_capacity(4096);
                let mut cdn_deb822_style = String::with_capacity(4096);
                let deb_vendor = match series {
                    "sarge" | "woody" | "potato" | "warty" => "",
                    _ => "[trusted=yes] ",
                };

                for src in enabled {
                    let enabled = true;

                    let (suite, site_left, site_suffix) =
                        get_debian_suite_and_site(src)?;
                    let (mirrors, keyring) =
                        get_debian_mirrors_and_keyring(site_left);

                    let mut deb_src = DebianSrc::builder()
                        .keyring(keyring)
                        .components(components)
                        .url(https_to_http(&mirrors[0]))
                        .url_suffix(site_suffix)
                        .suite(suite)
                        .enabled(enabled)
                        .src(src)
                        .deb_vendor(deb_vendor)
                        .build();

                    deb_src.update_debian_list(
                        &mut official_legacy_style,
                        &mut official_deb822_style,
                        &mirrors[1],
                        &mut cdn_legacy_style,
                        &mut cdn_deb822_style,
                    );
                }

                if let Some(disabled) = disabled_srcs {
                    for src in disabled {
                        let enabled = false;
                        let (suite, site_name, site_suffix) =
                            get_debian_suite_and_site(src)?;
                        let (mirrors, keyring) =
                            get_debian_mirrors_and_keyring(site_name);

                        let mut deb_src = DebianSrc::builder()
                            .keyring(keyring)
                            .components(components)
                            .url(https_to_http(&mirrors[0]))
                            .url_suffix(site_suffix)
                            .suite(suite)
                            .enabled(enabled)
                            .src(src)
                            .deb_vendor(deb_vendor)
                            .build();

                        deb_src.update_debian_list(
                            &mut official_legacy_style,
                            &mut official_deb822_style,
                            &mirrors[1],
                            &mut cdn_legacy_style,
                            &mut cdn_deb822_style,
                        );
                    }
                }

                fs::write(mirror_dir.join("Official.list"), official_legacy_style)?;
                fs::write(mirror_dir.join("NJU.CN.list"), cdn_legacy_style)?;

                fs::write(
                    mirror_dir.join("Official.sources"),
                    official_deb822_style,
                )?;
                fs::write(mirror_dir.join("NJU.CN.sources"), cdn_deb822_style)?;
            }
        }
        create_src_list_link(mirror_dir)?;
        create_deb822_link(mirror_dir)?;
        Ok(())
    }
}

pub(crate) fn create_src_list_link(mirror_dir: &Path) -> io::Result<()> {
    let src_link = mirror_dir.join("sources.list");

    // link.exists() returns false when the link file points to a file that does not exist.
    if src_link.is_symlink() || src_link.exists() {
        fs::remove_file(&src_link)?;
    }
    std::os::unix::fs::symlink(
        "../../usr/local/etc/apt/mirrors/Official.list",
        src_link,
    )
}

fn create_deb822_link(mirror_dir: &Path) -> io::Result<()> {
    let deb822_link = mirror_dir.join("mirror.sources");
    if deb822_link.is_symlink() || deb822_link.exists() {
        fs::remove_file(&deb822_link)?;
    }
    std::os::unix::fs::symlink(
        "../../../usr/local/etc/apt/mirrors/Official.sources",
        deb822_link,
    )
}

fn get_debian_mirrors_and_keyring(
    site_name: &str,
) -> ([mirror::Mirror<'_>; 2], &str) {
    let mirrors = get_debian_mirrors(site_name);
    let keyring = get_debian_keyring(site_name);
    (mirrors, keyring)
}
/**
```no_run
"debian-archive/debian/ potato" => {
    suite: potato,
    site_name: debian-archive,
    site_suffix: debian,
}

"debian-ports/ sid" => {
    suite: sid,
    site_name: debian-ports,
    site_suffix: ""
}
```
*/
fn get_debian_suite_and_site(
    src: &str,
) -> Result<(&str, &str, &str), anyhow::Error> {
    let (src_left, suite) = src
        .split_once(' ')
        .ok_or(anyhow::Error::msg("Sources must contain space"))?;
    let (site_left, site_right) = src_left
        .split_once('/')
        .ok_or(anyhow::Error::msg("left must contain /"))?;
    Ok((suite, site_left, site_right))
}

#[derive(Getters, TypedBuilder, Debug)]
struct DebianSrc<'a> {
    url: String,
    url_suffix: &'a str,
    src: &'a str,
    suite: &'a str,
    components: &'a str,
    keyring: &'a str,
    deb_vendor: &'a str,
    enabled: bool,
}

impl<'a> DebianSrc<'a> {
    fn one_line_str(&self) -> String {
        let Self {
            url,
            url_suffix,
            suite,
            components,
            enabled,
            deb_vendor,
            ..
        } = self;

        let prefix = if *enabled { "" } else { "# " };

        format!("{prefix}deb {deb_vendor}{url}{url_suffix} {suite} {components}\n")
    }

    fn one_line_debsrc_str(&self) -> String {
        let Self {
            url,
            url_suffix,
            suite,
            components,
            deb_vendor,
            ..
        } = self;
        format!("# deb-src {deb_vendor}{url}{url_suffix} {suite} {components}\n\n")
    }

    fn deb822_str(&self) -> String {
        let Self {
            url,
            url_suffix,
            suite,
            components,
            enabled,
            keyring,
            src,
            ..
        } = self;

        let yes_or_no = if *enabled { "yes" } else { "no" };

        let http_url = match (url, suite) {
            (u, &"sid" | &"experimental")
                if u.contains("deb.debian.org/debian/") =>
            {
                get_static_debian_snapshot_url(false).as_ref()
            }
            (u, &"sid" | &"experimental")
                if u.contains("deb.debian.org/debian-ports/") =>
            {
                get_static_debian_snapshot_url(true).as_ref()
            }
            _ => None,
        }
        .map_or(url.as_str(), |x| x.as_str());

        let https_url = http_str_to_https(url);

        format!(
            r##"# Name: {src}
# yes or no
Enabled: {yes_or_no}
# Types: deb deb-src
Types: deb
# URIs: {https_url}{url_suffix}
URIs: {http_url}{url_suffix}
Suites: {suite}
Components: {components}
Signed-By: {keyring}
Trusted: yes
# When using official source, recommended => yes;
#      using mirror => no;
#      using snapshot => no.
Check-Valid-Until: no
# Allow-Insecure: no

"##
        )
    }

    fn update_debian_list(
        &mut self,
        official_legacy_style: &mut String,
        official_deb822_style: &mut String,
        cdn_mirror: &mirror::Mirror<'_>,
        cdn_legacy_style: &mut String,
        cdn_deb822_style: &mut String,
    ) {
        // official mirror:
        official_legacy_style.push_str(&self.one_line_str());
        official_legacy_style.push_str(&self.one_line_debsrc_str());

        official_deb822_style.push_str(&self.deb822_str());

        // cdn mirror:
        self.url = https_to_http(cdn_mirror);
        cdn_legacy_style.push_str(&self.one_line_str());
        cdn_legacy_style.push_str(&self.one_line_debsrc_str());
        cdn_deb822_style.push_str(&self.deb822_str());
    }
}

fn get_static_debian_snapshot_url(ports: bool) -> &'static Option<Url> {
    type OnceUrl = OnceLock<Option<Url>>;
    static U: OnceUrl = OnceLock::new();
    static PORTS: OnceUrl = OnceLock::new();

    if ports {
        PORTS.get_or_init(|| get_snapshot_url(ports))
    } else {
        U.get_or_init(|| get_snapshot_url(ports))
    }
}

fn get_snapshot_url(ports: bool) -> Option<Url> {
    let year = today_date().year();
    let month = today_date().month() as u8;
    let debian = if ports { "debian-ports" } else { "debian" };

    let mut url = static_debian_snapshot()
        .join(&format!("archive/{debian}/"))
        .expect("Invalid snapshot url");
    // const ERR_MSG: &str = "Failed to connect to the debian snapshot site";

    url.set_query(Some(&format!("year={year}&month={month}")));

    if !check_http_status(url.as_str()) {
        let (new_year, new_month) =
            if month == 1 { (year - 1, 12) } else { (year, month - 1) };

        url.set_query(Some(&format!("year={new_year}&month={new_month}")));
        if !check_http_status(url.as_str()) {
            return None;
        }
    }
    let html = run_and_get_stdout("curl", &["-L", url.as_str()]).ok()?;
    log::trace!("html: {html}");

    let mut child = Command::new("awk")
        .args([
            r#"-F""#,
            // r#"/="[0-9]+T[0-9]+Z\/"/ {print $2}"#,
            r#"/="[0-9]+T[0-9]+Z\/"/ {c = $2} END{print(c)}"#,
        ])
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .ok()?;

    child
        .stdin
        .as_mut()?
        .write_all(html.as_bytes())
        .ok()?;

    let out = child
        .wait_with_output()
        .ok()?
        .stdout;

    let snapshot_rfc3339 = String::from_utf8_lossy(&out)
        .trim()
        .to_owned();
    log::debug!("snapshot: {snapshot_rfc3339}");

    url.set_query(None);

    let mut snap_url = url
        .join(&snapshot_rfc3339)
        .ok()?;

    log::info!("snapshot url: {snap_url}");
    snap_url
        .set_scheme("http")
        .ok()?;

    Some(snap_url)
}

fn check_http_status(url: &str) -> bool {
    let Ok(out) = run_and_get_stdout("curl", &["-LI", url]) else {
        return false;
    };

    out.lines()
        .any(|x| x.contains(" 200 OK"))
}

fn get_debian_components(components: Option<&str>) -> &str {
    components.unwrap_or(components::OLD_DEBIAN)
}

fn get_debian_keyring(site_left: &str) -> &str {
    match site_left {
        "debian-ports" => "/usr/share/keyrings/debian-ports-archive-keyring.gpg",
        "debian-elts" | "debian-elts-official" => {
            "/etc/apt/trusted.gpg.d/freexian-archive-extended-lts.gpg"
        }
        _ => "/usr/share/keyrings/debian-archive-keyring.gpg",
    }
}

fn get_debian_mirrors(site_left: &str) -> [mirror::Mirror<'_>; 2] {
    match site_left {
        "debian-elts" | "debian-elts-official" => mirror::debian_elts::mirrors(),
        "debian-debug" => mirror::debian_debug::mirrors(),
        "debian-archive" => mirror::debian_archive::root_mirrors(),
        "debian-ports" => mirror::debian_ports::mirrors(),
        "debian-security" => mirror::debian_security::mirrors(),
        _ => mirror::debian::mirrors(),
    }
}

fn ubuntu_one_line_style(suite: &str, url: &str) -> String {
    let components = components::UBUNTU;
    let deb_vendor = match suite {
        "warty" => "",
        _ => "[trusted=yes] ",
    };

    format!(
        r##"
deb {deb_vendor}{url} {suite} {components}
# deb-src {deb_vendor}{url} {suite} {components}

deb {deb_vendor}{url} {suite}-updates {components}
# deb-src {deb_vendor}{url} {suite}-updates {components}

deb {deb_vendor}{url} {suite}-backports {components}
# deb-src {deb_vendor}{url} {suite}-backports {components}

deb {deb_vendor}{url} {suite}-security {components}
# deb-src {deb_vendor}{url} {suite}-security {components}

# --------
# Disabled
# deb [trusted=yes] {url} {suite}-proposed {components}
# deb-src [trusted=yes] {url} {suite}-proposed {components}
"##
    )
}

fn ubuntu_deb822_style(suite: &str, url: &str, name: &str) -> String {
    let components = components::UBUNTU;
    let url_cmt = match url {
        ubuntu::OFFICIAL => {
            Cow::from("# URIs: mirror://mirrors.ubuntu.com/mirrors.txt")
        }
        ubuntu_ports::OFFICIAL => Cow::from(
            r##"# get ubuntu-ports mirror:  curl -L mirrors.ubuntu.com/mirrors.txt | awk '{ sub(/ubuntu(\/)?$/, "ubuntu-ports/"); sprintf("curl -sI %s", $0) | getline status; if (status ~ /^HTTP.* 200 /) print}'"##,
        ),
        _ => Cow::from(format!("# URIs: {}", http_str_to_https(url))),
    };

    format!(
        r##"# Name: ubuntu {suite} ({name})
# yes or no
Enabled: yes
# Types: deb deb-src
Types: deb
# {url_cmt}
URIs: {url}
# Suites: {suite} {suite}-updates {suite}-backports {suite}-security {suite}-proposed
Suites: {suite} {suite}-updates {suite}-backports {suite}-security
Components: {components}
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
Trusted: yes
# When using official source, recommended => yes;
#      using mirror => no.
Check-Valid-Until: no
# Allow-Insecure: no

"##
    )
}

fn legacy_src_list_path(
    m: &mirror::Mirror<'_>,
    mirror_dir: &Path,
    name: &str,
) -> PathBuf {
    let (region_prefix, region) = match m.get_region() {
        Some(r) => (".", r),
        _ => ("", &""),
    };
    mirror_dir.join(format!("{name}{region_prefix}{region}.list"))
}

pub(crate) fn https_to_http(m: &mirror::Mirror<'_>) -> String {
    https_str_to_http(m.get_url())
}

fn https_str_to_http(https: &str) -> String {
    https.replacen("https://", "http://", 1)
}

fn http_str_to_https(http: &str) -> String {
    http.replacen("http://", "https://", 1)
}

impl<'r> Default for Repository<'r> {
    fn default() -> Self {
        let dft = Default::default();

        Self::builder()
            .codename(dft)
            .series(dft)
            .version(dft)
            .arch(dft)
            .build()
    }
}

pub(crate) type NormalRepos = TinyVec<[String; 2]>;

pub(crate) type MainRepos = TinyVec<[repo_map::MainRepo; 2]>;

impl<'r> Repository<'r> {
    pub(crate) const REG_URI: &'static str = "reg.tmoe.me:2096";
    pub(crate) const GHCR_URI: &'static str = "ghcr.io";

    pub(crate) fn oci_platform(&self) -> &str {
        get_oci_platform(self.arch)
    }

    pub(crate) fn base_name(&self) -> String {
        let opt_prefix = |opt: Option<&str>| match opt {
            Some(d) if !d.trim().is_empty() => Cow::from(format!("_{}", d)),
            _ => Cow::from(""),
        };

        let opt_date = match self.date.trim() {
            "1900-01-01" | "" => Cow::from(""),
            d => Cow::from(format!("_{}", d)),
        };

        format!(
            "{}_{}_{}{}{}",
            self.version,
            self.series,
            self.arch,
            opt_prefix(self.tag),
            opt_date,
        )
    }

    pub(crate) fn opt_tag_suffix(&self) -> Cow<str> {
        match self.tag {
            Some(t) => Cow::from(format!("-{t}")),
            _ => Cow::from(""),
        }
    }

    pub(crate) fn opt_tag_prefix(&self) -> Cow<str> {
        match self.tag {
            Some(t) => Cow::from(format!("{t}-")),
            _ => Cow::from(""),
        }
    }

    /// -> `[ghcr.io/xx/yy, ghcr.io/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn ghcr_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            series,
            arch,
            version,
            ..
        } = self;

        [
            format!(
                // "{}-{}{}",
                // ghcr.io/2cd/debian:potato-x86-base OR 2cd/debian:bo-x86
                "{}/{}/{}:{}-{}{}",
                uri, owner, project, series, arch, suffix
            ),
            format!(
                // ghcr: "{}-{}{}",
                // ghcr.io/2cd/debian:potato-x86-base OR ghcr.io/2cd/debian:bo-x86
                "{}/{}/{}:{}-{}{}",
                uri, owner, project, version, arch, suffix
            ),
        ]
        .into()
    }

    pub(crate) fn ghcr_date_tagged_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;

        let Self {
            owner,
            project,
            // series,
            // version,
            arch,
            ..
        } = self;

        [
            format!(
                // ghcr.io/2cd/debian-sid:x64 OR x64-base
                "{uri}/{owner}/{project}:{arch}{suffix}",
            ),
            format!(
                // ghcr.io/2cd/debian-sid:x64-2024-01-01
                // OR: x64-base-2024-01-01
                "{uri}/{owner}/{project}:{arch}{suffix}-{today}",
                today = logger::today()
            ),
        ]
        .into()
    }

    pub(crate) fn get_reg_date_tagged_owner(&self) -> &str {
        match *self.get_project() {
            "debian-sid" => "debian",
            "ubuntu-dev" => "ubuntu",
            p => p,
        }
    }

    pub(crate) fn reg_date_tagged_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::REG_URI;

        let Self {
            // owner,
            // project,
            series,
            // version,
            arch,
            ..
        } = self;

        let owner = self.get_reg_date_tagged_owner();

        [
            format!(
                // REG_URI/debian/sid:x64 OR x64-base
                "{uri}/{owner}/{series}:{arch}{suffix}",
            ),
            format!(
                // REG_URI/debian/sid:x64-2024-01-01
                "{uri}/{owner}/{series}:{arch}{suffix}-{date}",
                date = logger::today(),
            ),
        ]
        .into()
    }

    /// -> `[reg.tmoe.me/xx/yy, reg.tmoe.me/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn reg_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::REG_URI;
        let Self {
            project,
            series,
            version,
            arch,
            ..
        } = self;

        [
            format!(
                // REG_URI/debian/potato:x86-base OR REG_URI/debian/bo-x86
                "{}/{}/{}:{}{}",
                uri, project, series, arch, suffix
            ),
            format!(
                // REG_URI/debian/2.2:x86-base OR REG_URI/debian/1.3:x86
                "{}/{}/{}:{}{}",
                uri, project, version, arch, suffix
            ),
        ]
        .into()
    }

    /// e.g. -> `[MainRepo::Reg(REG_URI/debian/bo:latest), MainRepo::Reg(REG_URI/debian/1.3:latest)]`
    pub(crate) fn reg_main_repos(&self) -> MainRepos {
        let uri = Self::REG_URI;
        let tag = self.tag.unwrap_or("latest");
        let Self {
            project,
            series,
            version,
            ..
        } = self;

        [
            // REG_URI/debian/potato:base OR REG_URI/debian/bo:latest
            format!("{}/{}/{}:{}", uri, project, series, tag),
            // REG_URI/debian/2.2:base OR REG_URI/debian/1.3:latest
            format!("{}/{}/{}:{}", uri, project, version, tag),
        ]
        .map(repo_map::MainRepo::Reg)
        .into()
    }

    pub(crate) fn ghcr_main_repos(&self) -> MainRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            series,
            version,
            ..
        } = self;

        [
            // ghcr.io/2cd/debian:potato-base OR ghcr.io/2cd/debian:bo
            format!("{}/{}/{}:{}{}", uri, owner, project, series, suffix),
            // ghcr.io/2cd/debian:2.2-base OR ghcr.io/2cd/debian:1.3
            format!("{}/{}/{}:{}{}", uri, owner, project, version, suffix),
        ]
        .map(repo_map::MainRepo::Ghcr)
        .into()
    }
    // date_tagged_repos

    pub(crate) fn reg_main_date_tagged_repos(&self) -> MainRepos {
        let uri = Self::REG_URI;
        let tag = self.tag.unwrap_or("latest");
        let series = self.get_series();
        let prefix = self.opt_tag_prefix();

        let owner = self.get_reg_date_tagged_owner();

        // REG_URI/debian/sid:tag
        // REG_URI/debian/sid:2024-01-01 OR base-2024-01-01
        [
            format!("{uri}/{owner}/{series}:{tag}"),
            format!(
                "{uri}/{owner}/{series}:{prefix}{date}",
                date = logger::today()
            ),
        ]
        .map(repo_map::MainRepo::Reg)
        .into()
    }

    pub(crate) fn ghcr_main_date_tagged_repos(&self) -> MainRepos {
        let prefix = self.opt_tag_prefix();

        let tag = self.tag.unwrap_or("latest");

        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            // series,
            // version,
            ..
        } = self;

        // ghcr.io/2cd/debian-sid:TAG, if TAG is empty => latest
        // ghcr.io/2cd/debian-sid:2024-01-01 OR base-2024-01-01
        [
            format!("{uri}/{owner}/{project}:{tag}"),
            format!(
                "{uri}/{owner}/{project}:{prefix}{date}",
                date = logger::today()
            ),
        ]
        .map(repo_map::MainRepo::Ghcr)
        .into()
    }
}
