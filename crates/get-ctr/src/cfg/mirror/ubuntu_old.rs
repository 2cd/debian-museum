// https://old-releases.ubuntu.com/ubuntu/
// https://mirrors.ustc.edu.cn/ubuntu-old-releases/ubuntu/
use crate::cfg::mirror::{Mirror, MirrorVariant};

/// Creates a new instance of Mirror (UbuntuOld).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::UbuntuOld,
    }
}

const fn ustc<'m>() -> Mirror<'m> {
    new_mirror(
        "USTC",
        "https://mirrors.ustc.edu.cn/ubuntu-old-releases/ubuntu/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", "https://old-releases.ubuntu.com/ubuntu/", None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), ustc()]
}
