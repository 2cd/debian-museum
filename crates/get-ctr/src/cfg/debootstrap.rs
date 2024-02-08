use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
struct Tag {
    arch: String,
    #[serde(rename = "deb-arch")]
    deb_arch: String,

    #[serde(flatten)]
    source: Source,
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(strip_option, into)))]
struct Source {
    src: Option<String>,
    sources: Option<Vec<String>>,

    #[serde(rename = "disabled_sources")]
    disabled_sources: Option<Vec<String>>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
struct OS {
    name: String,
    version: String,
    codename: String,
    series: String,
    date: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    compoents: Option<String>,

    #[serde(rename = "no-minbase")]
    no_minbase: bool,

    #[serde(rename = "deb822-format", default = "yes")]
    #[builder(default = true)]
    deb822_format: bool,

    #[serde(rename = "deb-architectures")]
    deb_architectures: Vec<String>,

    tag: Vec<Tag>,

    #[serde(flatten)]
    source: Source,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct DebootstrapCfg {
    os: Vec<OS>,
}

const fn yes() -> bool {
    true
}
