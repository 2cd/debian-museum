use crate::cfg::mirror::{Mirror, MirrorVariant};

/// Creates a new instance of Mirror (DebianELTS).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::DebianELTS,
    }
}

const fn nju<'m>() -> Mirror<'m> {
    new_mirror("NJU", "https://mirrors.nju.edu.cn/debian-elts/", Some("CN"))
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://deb.freexian.com/extended-lts/", None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}

pub(crate) const fn include_pkgs() -> &'static str {
    // "freexian-archive-keyring,ca-certificates,apt-transport-https"
    "freexian-archive-keyring,ca-certificates"
}
