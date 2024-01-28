mod cfg;
mod logger;

use log::{debug, info};
use std::{env, fs, io, path::Path};

const DISK_RON: &str = "assets/ci/base/disk.v1.ron";

fn main() -> anyhow::Result<()> {
    logger::init();

    #[cfg(debug_assertions)]
    set_workdir()?;

    let ron = Path::new(DISK_RON);
    debug!("current dir: {:?}", env::current_dir());

    log::trace!("parsing the ron cfg: {:?}", ron);
    // log::info!("{s}");
    let disk_cfg = ron::de::from_bytes::<cfg::DiskV1>(&fs::read(ron)?)?;

    let os = disk_cfg.get_os();
    // dbg!(os);
    for o in os {
        for disk in o.get_disk() {
            if disk.get_arch() == "armel" {
                dbg!(o.get_codename());
            }
        }
    }
    Ok(())
}

fn set_workdir() -> io::Result<()> {
    if env::current_dir()?.ends_with("crates/build-rootfs") {
        info!("set current dir to ../..");
        env::set_current_dir("../..")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set_workdir;

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
        use ron::ser::PrettyConfig;

        // logger::init();
        set_workdir()?;
        let path = Path::new(DISK_RON).with_extension("toml");

        dbg!(&path);

        let cfg = toml::from_str::<cfg::DiskV1>(&fs::read_to_string(path)?)?;

        let ron_str = ron::ser::to_string_pretty(
            &cfg,
            PrettyConfig::default()
                .enumerate_arrays(true)
                .depth_limit(4),
        )?;

        fs::write(DISK_RON, ron_str)?;

        Ok(())
    }
}
