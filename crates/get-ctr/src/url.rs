use crate::cfg::{
    self,
    mirror::{static_debian_archive_mirrors, Mirror, MirrorVariant},
};
use std::{env, sync::OnceLock};
use url::{ParseError, Url};

type UrlResult = Result<Url, ParseError>;

fn is_cn() -> bool {
    static B: OnceLock<bool> = OnceLock::new();
    *B.get_or_init(|| env::var("LANG").is_ok_and(|x| x.contains("CN")))
}

pub(crate) fn find_mirror_url(
    mirrors: &[Mirror],
    variant: MirrorVariant,
) -> UrlResult {
    let m = mirrors
        .iter()
        .filter(|x| x.get_variant() == &variant)
        .find(|x| match x.get_region() {
            None if is_cn() => false,
            Some("CN") => is_cn(),
            _ => true,
        })
        .ok_or(ParseError::EmptyHost)?;

    log::debug!("mirror: {m:?}");

    Url::parse(m.get_url())
}

pub(crate) fn debian_archive() -> UrlResult {
    log::debug!("finding the mirror url from static debian archive mirrors.");
    find_mirror_url(
        static_debian_archive_mirrors(),
        MirrorVariant::DebianArchive,
    )
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
