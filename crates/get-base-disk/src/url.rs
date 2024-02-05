use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

use crate::cfg::{self, disk::DiskV1};

#[derive(Deref, Serialize, Deserialize, From, Debug)]
#[from(forward)]
#[serde(transparent)]
pub(crate) struct ArchiveUrl(Url);

impl Default for ArchiveUrl {
    fn default() -> Self {
        // https://mirrors.nju.edu.cn/debian-archive/debian/dists/
        Url::parse("https://archive.debian.org/debian/dists/")
            .expect("Invalid url")
            .into()
    }
}

impl DiskV1 {
    pub(crate) fn find_mirror_url(&self) -> &Url {
        let is_cn = env::var("LANG").is_ok_and(|x| x.contains("CN"));

        self.get_mirror()
            .iter()
            .find(|x| match x.get_region().as_deref() {
                None if is_cn => false,
                Some("CN") => is_cn,
                _ => true,
            })
            .expect("Empty Mirror")
            .get_url()
    }
}

pub(crate) fn add_slash(x: &str) -> &str {
    if x.ends_with('/') {
        return "";
    }
    "/"
}

pub(crate) fn concat_url_path(
    url: &mut String,
    os: &cfg::disk::OS,
    disk: &cfg::disk::Disk,
) {
    let (op, dp, tgz) = (os.get_path(), disk.get_path(), os.get_base_tgz());
    url.clear();
    *url = format!(
        "{op}{o_sep}{dp}{d_sep}{tgz}",
        o_sep = add_slash(op),
        d_sep = add_slash(dp)
    );
}
