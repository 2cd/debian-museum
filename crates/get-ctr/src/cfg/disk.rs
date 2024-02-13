// use crate::cfg::mirror::Mirror;
use getset::Getters;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

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
    patch: Option<OsPatch>,
    disk: Vec<Disk>,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct OsPatch {
    #[serde(rename = "add-src-mirrors")]
    add_src_mirrors: bool,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, derive_more::Deref)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct DiskV1 {
    #[deref]
    os: Vec<OS>,
}

impl DiskV1 {
    pub(crate) fn deser() -> anyhow::Result<Self> {
        crate::dir::set_static_workdir();

        let content = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/old_old_debian/disk.v1.ron"
        ));

        debug!("deserializing the disk cfg");
        let cfg = ron::from_str(content)?;
        debug!("complete");

        trace!("cfg: {:?}", cfg);
        Ok(cfg)
    }
}
