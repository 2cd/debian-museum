mod debian_archive;
pub(crate) use debian_archive::static_mirrors as static_debian_archive_mirrors;

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

    UbuntuPorts,
    Ubuntu,
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
