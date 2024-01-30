use crate::url::ArchiveUrl;
use getset::Getters;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct Disk {
    arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    deb_arch: Option<String>,
    date: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct OS {
    codename: String,
    version: String,
    base_tgz: String,
    date: String,
    path: String,
    disk: Vec<Disk>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct Mirror {
    name: String,
    region: Option<String>,
    global: bool,
    url: ArchiveUrl,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct DiskV1 {
    mirror: Vec<Mirror>,
    os: Vec<OS>,
}

impl DiskV1 {
    pub(crate) const DISK_RON: &'static str = "assets/ci/base/disk.v1.ron";

    pub(crate) fn deser() -> anyhow::Result<Self> {
        let ron = Path::new(Self::DISK_RON);

        debug!("deserializing the ron cfg: {:?}", ron);
        let cfg = ron::de::from_bytes::<Self>(&fs::read(ron)?)?;
        debug!("complete");

        trace!("cfg: {:?}", cfg);
        Ok(cfg)
    }
}
