use std::str::FromStr;

use getset::Getters;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use url::Url;

use crate::DISK_RON;
#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
pub(crate) struct Disk {
    arch: String,
    date: String,
    path: String,
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
pub(crate) struct OS {
    codename: String,
    version: String,
    base_tgz: String,
    date: String,
    path: String,
    disk: Vec<Disk>,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
#[getset(get = "pub(crate) with_prefix")]
struct Mirror {
    name: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    global: bool,
    // #[serde(default = "debian_archive_url")]
    url: Url,
}

impl Default for Mirror {
    fn default() -> Self {
        Self {
            name: Default::default(),
            region: None,
            global: false,
            url: debian_archive_url(),
        }
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Default)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct DiskV1 {
    mirror: Vec<Mirror>,
    os: Vec<OS>,
}

fn debian_archive_url() -> Url {
    // https://mirrors.nju.edu.cn/debian-archive/debian/dists/
    Url::from_str("https://archive.debian.org/debian/dists/").expect("Invalid url")
}

pub(crate) fn parse() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    crate::set_workdir()?;

    let ron = Path::new(DISK_RON);
    debug!("current dir: {:?}", env::current_dir());

    log::trace!("parsing the ron cfg: {:?}", ron);
    let cfg = ron::de::from_bytes::<DiskV1>(&fs::read(ron)?)?;

    let dist_url = get_dist_url(&cfg);

    dbg!(dist_url);

    Ok(())
}

fn get_dist_url(cfg: &DiskV1) -> &Url {
    let is_cn = env::var("LANG").is_ok_and(|x| x.contains("CN"));

    cfg.get_mirror()
        .iter()
        .find(|x| match x.get_region().as_deref() {
            None if is_cn => false,
            Some("CN") => is_cn,
            _ => true,
        })
        .expect("Empty Mirror")
        .get_url()
}
