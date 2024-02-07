use getset::Getters;
use std::borrow::Cow;
use tinyvec::TinyVec;
use typed_builder::TypedBuilder;
use url::Url;

use crate::{
    cfg::disk::OsPatch,
    docker::{get_oci_platform, repo_map},
};

#[derive(Getters, TypedBuilder, Debug)]
#[getset(get = "pub(crate) with_prefix")]
#[builder(field_defaults(default))]
pub(crate) struct Repository<'r> {
    #[builder(default = "2cd")]
    owner: &'r str,
    #[builder(default = "debian")]
    project: &'r str,

    #[builder(!default, setter(transform = |s: &str| s.to_ascii_lowercase()))]
    codename: String,

    #[builder(!default)]
    version: &'r str,
    #[builder(!default)]
    arch: &'r str,

    tag: Option<&'r str>,

    #[builder(default = "1900-01-01")]
    date: &'r str,

    #[builder(setter(strip_option))]
    url: Option<Url>,

    #[builder(setter(strip_option))]
    title_date: Option<&'r str>,

    #[builder(setter(into))]
    patch: Option<&'r OsPatch>,
}

impl<'r> Default for Repository<'r> {
    fn default() -> Self {
        let dft = Default::default();

        Self::builder()
            .codename(dft)
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
            self.codename,
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

    /// -> `[ghcr.io/xx/yy, ghcr.io/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn ghcr_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            codename,
            arch,
            version,
            ..
        } = self;

        [
            format!(
                // "{}-{}{}",
                // ghcr.io/2cd/debian:potato-x86-base OR 2cd/debian:bo-x86
                "{}/{}/{}:{}-{}{}",
                uri, owner, project, codename, arch, suffix
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

    /// -> `[reg.tmoe.me/xx/yy, reg.tmoe.me/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn reg_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::REG_URI;
        let Self {
            project,
            codename,
            version,
            arch,
            ..
        } = self;

        [
            format!(
                // REG_URI/debian/potato:x86-base OR REG_URI/debian/bo-x86
                "{}/{}/{}:{}{}",
                uri, project, codename, arch, suffix
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
            codename,
            version,
            ..
        } = self;

        [
            // REG_URI/debian/potato:base OR REG_URI/debian/bo:latest
            format!("{}/{}/{}:{}", uri, project, codename, tag),
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
            codename,
            version,
            ..
        } = self;

        [
            // ghcr.io/2cd/debian:potato-base OR ghcr.io/2cd/debian:bo
            format!("{}/{}/{}:{}{}", uri, owner, project, codename, suffix),
            // ghcr.io/2cd/debian:2.2-base OR ghcr.io/2cd/debian:1.3
            format!("{}/{}/{}:{}{}", uri, owner, project, version, suffix),
        ]
        .map(repo_map::MainRepo::Ghcr)
        .into()
    }
}
