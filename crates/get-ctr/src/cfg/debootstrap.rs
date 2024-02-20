use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;
use url::Url;

pub(crate) const SCRIPT_DIR: &str = "/usr/share/debootstrap/scripts/";

use crate::{
    cfg::{components, mirror},
    url::find_mirror_url,
};

pub(crate) const DEBIAN_RON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/debootstrap/debian.ron"
));

pub(crate) const UBUNTU_RON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/debootstrap/ubuntu.ron"
));

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder, Clone)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct Tag {
    arch: String,
    #[serde(rename = "deb-arch")]
    deb_arch: String,

    source: Source,
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder, Clone)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(strip_option, into)))]
pub(crate) struct Source {
    src: Option<String>,
    enabled: Option<Vec<String>>,

    // #[serde(rename = "disabled-sources")]
    disabled: Option<Vec<String>>,
}

impl Source {
    pub(crate) fn disabled_srcs_owned(&self) -> Option<Vec<String>> {
        self.get_disabled().to_owned()
    }
}

#[derive(Getters, Debug, TypedBuilder, Clone)]
#[getset(get = "pub(crate) with_prefix")]
pub(crate) struct DebootstrapSrc {
    url: Url,

    #[builder(setter(into))]
    components: String,

    #[builder(setter(into))]
    suite: String,

    #[builder(default)]
    include_pkgs: Option<&'static str>,
    //
    // #[builder(default)]
    // exclude_pkgs: Option<&'static str>,
}

impl Source {
    pub(crate) fn debootstrap_src(&self, suite: &str) -> Option<DebootstrapSrc> {
        let Self {
            src,
            enabled: sources,
            ..
        } = self;
        log::debug!("sources: {sources:?}");

        match (src, sources) {
            (Some(src), _) => {
                let (uuu_mirrors, include_pkgs) = match src.as_str() {
                    "ubuntu" => (
                        mirror::ubuntu::mirrors(),
                        Some(mirror::ubuntu::include_pkgs()),
                    ),
                    "ubuntu-ports" => (
                        mirror::ubuntu_ports::mirrors(),
                        Some(mirror::ubuntu_ports::include_pkgs()),
                    ),
                    // ubuntu-old-releases
                    _ => (mirror::ubuntu_old::mirrors(), None),
                };
                find_mirror_url(&uuu_mirrors)
                    .ok()
                    .map(|url| {
                        DebootstrapSrc::builder()
                            .url(url)
                            .components(components::UBUNTU_BOOTSTRAP)
                            .suite(suite)
                            .include_pkgs(include_pkgs)
                            .build()
                    })
            }
            (_, Some(srcs)) => {
                let (site_left, suite) = srcs.first()?.split_once(' ')?;
                log::debug!("site_left: {site_left}, suite: {suite}");
                let mirror_name = site_left.split('/').next()?;

                match mirror_name {
                    "debian-archive" => {
                        find_mirror_url(&mirror::debian_archive::deb_mirrors())
                            .map(|url| (url, None))
                            .ok()
                    }
                    "debian-elts" => {
                        find_mirror_url(&mirror::debian_elts::mirrors())
                            .map(|url| {
                                (url, Some(mirror::debian_elts::include_pkgs()))
                            })
                            .ok()
                    }
                    "debian-elts-official" => mirror::debian_elts::mirrors()
                        .into_iter()
                        .find(|x| x.get_name() == &"Official")
                        .map(|mirror| {
                            (
                                Url::parse(mirror.get_url())
                                    .expect("Invalid ELTS URL"),
                                Some(mirror::debian_elts::include_pkgs()),
                            )
                        }),
                    "debian-ports" => {
                        find_mirror_url(&mirror::debian_ports::mirrors())
                            .map(|url| {
                                (url, Some(mirror::debian_ports::include_pkgs()))
                            })
                            .ok()
                    }
                    _ => find_mirror_url(&mirror::debian::mirrors())
                        .map(|url| (url, Some(mirror::debian::include_pkgs())))
                        .ok(),
                }
                // .ok()
                .map(|(url, pkgs)| {
                    DebootstrapSrc::builder()
                        .url(url)
                        .components(components::DEBIAN_BOOTSTRAP)
                        .suite(suite)
                        .include_pkgs(pkgs)
                        .build()
                })
            }
            _ => None,
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct OS {
    name: String,
    version: String,
    codename: String,
    series: String,
    date: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<String>,

    #[serde(rename = "no-minbase")]
    no_minbase: bool,

    #[serde(rename = "deb822-format", default = "yes")]
    #[builder(default = true)]
    deb822_format: bool,

    #[serde(rename = "deb-architectures")]
    deb_architectures: Vec<String>,

    tag: Vec<Tag>,

    source: Source,
}

#[derive(
    Serialize, Deserialize, Debug, Default, TypedBuilder, derive_more::Deref,
)]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct Cfg {
    #[deref]
    pub(crate) os: Vec<OS>,
}

const fn yes() -> bool {
    true
}
