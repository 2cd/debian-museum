use crate::cfg::mirror::{Mirror, MirrorVariant};
use std::sync::OnceLock;

/// Creates a new instance of Mirror (debian archive).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::DebianArchive,
    }
}

const fn nju<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/debian-archive/debian/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://archive.debian.org/debian/", None)
}

pub(crate) fn static_mirrors() -> &'static [Mirror<'static>; 2] {
    static M: OnceLock<[Mirror; 2]> = OnceLock::new();
    M.get_or_init(|| [official(), nju()])
}
