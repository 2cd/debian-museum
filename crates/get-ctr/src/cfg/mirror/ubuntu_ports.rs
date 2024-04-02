// mirror://mirrors.ubuntu.com/mirrors.txt
use crate::cfg::mirror::{self, Mirror, MirrorVariant};
pub(crate) use mirror::include_pkgs;
pub(crate) const OFFICIAL: &str = "http://ports.ubuntu.com/ubuntu-ports/";

/// Creates a new instance of Mirror (UbuntuPorts).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::UbuntuPorts,
    }
}

const fn nju<'m>() -> Mirror<'m> {
    new_mirror(
        "NJU",
        "https://mirrors.nju.edu.cn/ubuntu-ports/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror("Official", OFFICIAL, None)
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), nju()]
}
