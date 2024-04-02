use crate::cfg::mirror::{Mirror, MirrorVariant};
const OFFICIAL: &str = "https://deb.debian.org/debian-ports/";

/// Creates a new instance of Mirror (DebianPorts).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::DebianPorts,
    }
}

const fn nju<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/debian-ports/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", OFFICIAL, None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}

pub(crate) const fn include_pkgs() -> &'static str {
    "debian-ports-archive-keyring,ca-certificates"
}
