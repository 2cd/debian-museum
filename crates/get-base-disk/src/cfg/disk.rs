use crate::url::ArchiveUrl;
use getset::Getters;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct Disk {
    pub(crate) arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) deb_arch: Option<String>,
    pub(crate) date: String,
    pub(crate) path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tag: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct OS {
    pub(crate) codename: String,
    pub(crate) version: String,
    pub(crate) base_tgz: String,
    pub(crate) date: String,
    pub(crate) path: String,
    pub(crate) disk: Vec<Disk>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct Mirror {
    pub(crate) name: String,
    pub(crate) region: Option<String>,
    pub(crate) global: bool,
    pub(crate) url: ArchiveUrl,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct DiskV1 {
    pub(crate) mirror: Vec<Mirror>,
    pub(crate) os: Vec<OS>,
}

impl DiskV1 {
    pub(crate) const DISK_RON: &'static str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ci/old_old_debian/disk.v1.ron"
    ));

    pub(crate) fn deser() -> anyhow::Result<Self> {
        crate::dir::set_static_workdir();

        debug!("deserializing the ron cfg");
        let cfg = ron::from_str(Self::DISK_RON)?;
        debug!("complete");

        trace!("cfg: {:?}", cfg);
        Ok(cfg)
    }
}
