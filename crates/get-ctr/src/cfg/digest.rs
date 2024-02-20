use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::PathBuf;
use typed_builder::TypedBuilder;
use url::Url;

use crate::task::old_old_debian::docker_task::MainRepoDigests;

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(into)))]
pub(crate) struct DockerMirror {
    #[builder(!default)]
    name: String,
    #[builder(!default)]
    repositories: MainRepoDigests,

    #[serde(rename = "repo-digests")]
    repo_digests: Option<MainRepoDigests>,

    #[builder(setter(strip_option))]
    user: Option<String>,

    #[builder(setter(strip_option))]
    token: Option<String>,
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(into)))]
pub(crate) struct Docker {
    #[builder(default, setter(strip_option))]
    platform: Option<String>,

    #[serde(rename = "oci-platforms")]
    oci_platforms: Option<Vec<String>>,

    #[serde(rename = "repo-digests")]
    repo_digests: Option<MainRepoDigests>,

    #[builder(default, setter(strip_option))]
    cmt: Option<String>,

    mirror: Vec<DockerMirror>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct HashDigest {
    algorithm: String,
    hex: String,

    #[builder(default, setter(strip_option))]
    cmt: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Debug, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct FileMirror {
    #[builder(default)]
    name: String,

    #[builder(default = Url::parse("file:///").expect("Invalid URL") )]
    url: Url,

    #[builder(default, setter(strip_option))]
    cmt: Option<String>,
}

impl Default for FileMirror {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct FileSize {
    bytes: u64,
    readable: byteunit::ByteUnit,
    #[builder(default)]
    kib: Option<byteunit::ByteUnit>,
    #[builder(default)]
    mib: Option<byteunit::ByteUnit>,

    #[serde(rename = "tar-bytes")]
    tar_bytes: u64,
    #[serde(rename = "tar-readable")]
    tar_readable: byteunit::ByteUnit,

    #[builder(default, setter(strip_option))]
    cmt: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct Zstd {
    level: u8,
    #[builder(default = false)]
    #[serde(rename = "long-distance")]
    long_distance: bool,

    #[builder(default = false)]
    dict: bool,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct ArchiveFile {
    name: PathBuf,
    size: FileSize,

    #[serde(rename = "modified-time")]
    #[builder(default, setter(strip_option))]
    modified_time: Option<time::OffsetDateTime>,

    #[builder(setter(strip_option))]
    zstd: Option<Zstd>,

    digest: Vec<HashDigest>,
    mirror: Vec<FileMirror>,
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct DateTime {
    #[serde(rename = "build")]
    #[builder(default, setter(strip_option))]
    build_time: Option<time::OffsetDateTime>,

    #[serde(rename = "update")]
    #[builder(default, setter(strip_option))]
    update_time: Option<time::OffsetDateTime>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct MainTag {
    name: String,
    arch: String,
    datetime: DateTime,
    docker: Docker,
    file: ArchiveFile,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct OS {
    name: String,
    codename: String,

    #[builder(setter(strip_option))]
    series: Option<String>,
    version: String,

    docker: Docker,
    #[builder(default)]
    pub(crate) tag: Vec<MainTag>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct Digests {
    os: Vec<OS>,
}
