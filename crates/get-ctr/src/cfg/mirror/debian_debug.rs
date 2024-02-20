use crate::cfg::mirror::{Mirror, MirrorVariant};

/// Creates a new instance of Mirror (DebianDebug).
const fn new_mirror<'m>(
    name: &'m str,
    url: &'m str,
    region: Option<&'m str>,
) -> Mirror<'m> {
    Mirror {
        name,
        region,
        url,
        variant: MirrorVariant::DebianDebug,
    }
}

const fn byte_dance_volces<'m>() -> Mirror<'m> {
    new_mirror(
        "ByteDance-Volcengine",
        "https://mirrors.volces.com/debian-debug/",
        Some("CN"),
    )
}

const fn official<'m>() -> Mirror<'m> {
    new_mirror(
        "Official",
        "http://debug.mirrors.debian.org/debian-debug/",
        None,
    )
}

pub(crate) const fn mirrors() -> [Mirror<'static>; 2] {
    [official(), byte_dance_volces()]
}
