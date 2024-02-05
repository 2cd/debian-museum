use std::path::PathBuf;

use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;
use url::Url;

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(into, strip_option)))]
pub(crate) struct DockerMirror {
    #[builder(!default, setter(!strip_option))]
    name: String,
    #[builder(!default, setter(!strip_option))]
    repositories: Vec<String>,

    #[serde(rename = "repo-digest")]
    repo_digest: Option<String>,
    user: Option<String>,
    token: Option<String>,
}

#[skip_serializing_none]
#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(default, setter(into)))]
pub(crate) struct Docker {
    #[builder(default, setter(strip_option))]
    cmt: Option<String>,

    #[builder(default, setter(strip_option))]
    platform: Option<String>,

    #[serde(rename = "oci-platforms")]
    oci_platforms: Option<Vec<String>>,

    mirror: Vec<DockerMirror>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct HashDigest {
    algorithm: String,
    hex: String,
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
}

impl Default for FileMirror {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, TypedBuilder)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
#[builder(field_defaults(setter(into)))]
pub(crate) struct FileSize {
    bytes: u64,
    readable: byteunit::ByteUnit,
    #[serde(rename = "readable-kilo-binary-byte")]
    readable_kib: byteunit::ByteUnit,

    #[serde(rename = "tar-bytes")]
    tar_bytes: u64,
    #[serde(rename = "tar-readable")]
    tar_readable: byteunit::ByteUnit,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io, path::Path};

    #[test]
    fn deser_toml() -> anyhow::Result<()> {
        let file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp.todo/v1.toml");
        if !file.exists() {
            return Ok(());
        }

        let toml = toml::from_str::<Digests>(&fs::read_to_string(&file)?)?;
        dbg!(toml);
        Ok(())
    }
}
