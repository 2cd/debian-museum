use crate::cfg::mirror::{Mirror, MirrorVariant};

/// Creates a new instance of Mirror (debian).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::Debian,
    }
}

const fn nju<'m>() -> Mirror<'m> {
    new_mirror("NJU", "https://mirrors.nju.edu.cn/debian/", Some("CN"))
}

// const fn cloudflare<'m>() -> Mirror<'m> {
//     new_mirror("CloudFlare", "https://cloudflaremirrors.com/debian/", None)
// }

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://deb.debian.org/debian/", None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}

pub(crate) const fn include_pkgs() -> &'static str {
    "ca-certificates"
}
