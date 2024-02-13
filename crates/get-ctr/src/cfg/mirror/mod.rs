pub(crate) mod debian;
pub(crate) mod debian_archive;
pub(crate) mod debian_debug;
pub(crate) mod debian_elts;
pub(crate) mod debian_ports;
pub(crate) mod debian_security;

pub(crate) mod ubuntu;
pub(crate) mod ubuntu_old;
pub(crate) mod ubuntu_ports;

use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, derive_more::Display,
)]
pub(crate) enum MirrorVariant {
    #[display(fmt = "debian-archive")]
    DebianArchive,

    #[display(fmt = "debian")]
    Debian,

    #[display(fmt = "debian-ports")]
    DebianPorts,

    #[display(fmt = "debian-elts")]
    DebianELTS,

    #[display(fmt = "debian-debug")]
    DebianDebug,

    #[display(fmt = "debian-security")]
    DebianSecurity,

    #[display(fmt = "ubuntu-ports")]
    UbuntuPorts,

    #[display(fmt = "ubuntu")]
    Ubuntu,

    #[display(fmt = "ubuntu-old")]
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
    "ca-certificates,apt-transport-https"
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
