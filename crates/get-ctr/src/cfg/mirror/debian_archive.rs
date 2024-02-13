//! const_deb_mirrors:
//!     - archive.debian.org/debian/
//!
//! const_mirrors:
//!     - archive.debian.org/
use crate::cfg::mirror::{Mirror, MirrorVariant};

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

const fn nju_deb<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/debian-archive/debian/",
        Some("CN"),
    )
}

const fn official_deb<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://archive.debian.org/debian/", None)
}
const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://archive.debian.org/", None)
}
const fn nju<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/debian-archive/",
        Some("CN"),
    )
}

pub(crate) const fn deb_mirrors() -> [Mirror<'static>; 2] {
    [official_deb(), nju_deb()]
}

pub(crate) const fn root_mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}
