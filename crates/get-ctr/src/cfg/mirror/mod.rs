pub(crate) mod debian;
pub(crate) mod debian_archive;
pub(crate) mod debian_debug;
pub(crate) mod debian_elts;
pub(crate) mod debian_ports;
pub(crate) mod debian_security;

pub(crate) mod ubuntu;
pub(crate) mod ubuntu_old;
pub(crate) mod ubuntu_ports;

pub(crate) fn static_debian_snapshot() -> &'static Url {
    static U: OnceLock<Url> = OnceLock::new();
    U.get_or_init(|| {
        const HTTPS: &str = "https://snapshot.debian.org/";
        Url::parse(HTTPS).expect("Invalid URL")
    })
}

use std::sync::OnceLock;

use getset::Getters;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, derive_more::Display,
)]
pub(crate) enum MirrorVariant {
    #[display("debian-archive")]
    DebianArchive,

    #[display("debian")]
    Debian,

    #[display("debian-ports")]
    DebianPorts,

    #[display("debian-elts")]
    DebianELTS,

    #[display("debian-debug")]
    DebianDebug,

    #[display("debian-security")]
    DebianSecurity,

    #[display("ubuntu-ports")]
    UbuntuPorts,

    #[display("ubuntu")]
    Ubuntu,

    #[display("ubuntu-old")]
    UbuntuOld,
}

impl Default for MirrorVariant {
    fn default() -> Self {
        Self::DebianArchive
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Default, Clone, Copy)]
#[getset(get = "pub(crate) with_prefix")]
#[serde(default)]
pub(crate) struct Mirror<'m> {
    name: &'m str,

    region: Option<&'m str>,

    url: &'m str,

    variant: MirrorVariant,
}

pub(crate) const fn include_pkgs() -> &'static str {
    // "ca-certificates,apt-transport-https"
    "ca-certificates"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_variant() {
        let var = MirrorVariant::default();
        println!("{var}");

        let elts = MirrorVariant::DebianELTS;
        println!("{elts}");
    }
}
