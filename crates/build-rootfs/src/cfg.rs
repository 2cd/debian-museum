use std::str::FromStr;

use getset::Getters;
use serde::{Deserialize, Serialize};
use url::Url;

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
pub(crate) struct DiskV1 {
    #[serde(default = "debian_archive_url")]
    url: Url,
    #[serde(default)]
    os: Vec<OS>,
}

fn debian_archive_url() -> Url {
    Url::from_str("https://archive.debian.org/debian/dists/").expect("Invalid url")
}
