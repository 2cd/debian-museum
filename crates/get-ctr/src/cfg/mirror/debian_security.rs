use crate::cfg::mirror::{Mirror, MirrorVariant};

/// Creates a new instance of Mirror (DebianSecurity).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::DebianSecurity,
    }
}

// const fn cernet<'m>() -> Mirror<'m> {
//     new_mirror(
//         "Cernet",
//         "https://mirrors.cernet.edu.cn/debian-security/",
//         Some("CN"),
//     )
// }

const fn nju<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/debian-security/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://deb.debian.org/debian-security/", None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}
