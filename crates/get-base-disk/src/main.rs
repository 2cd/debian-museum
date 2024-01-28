mod cfg;
mod logger;
use std::{env, path::Path};

const DISK_RON: &str = "assets/ci/base/disk.v1.ron";

fn main() -> anyhow::Result<()> {
    logger::init();
    cfg::parse()?;

    Ok(())
}

#[cfg(debug_assertions)]
fn set_workdir() -> std::io::Result<()> {
    use log_l10n::get_pkg_name;

    if env::current_dir()?.ends_with(Path::new("crates").join(get_pkg_name!())) {
        log::info!("set current dir to ../..");
        env::set_current_dir("../..")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ron::extensions::Extensions;

    use super::*;
    use std::fs;

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
        use ron::ser::PrettyConfig;

        // logger::init();
        set_workdir()?;
        dbg!(env::current_dir());
        let path = Path::new(DISK_RON).with_extension("toml");
        dbg!(&path);

        let cfg = toml::from_str::<cfg::DiskV1>(&fs::read_to_string(path)?)?;

        let ron_str = ron::ser::to_string_pretty(
            &cfg,
            PrettyConfig::default()
                .enumerate_arrays(true)
                .depth_limit(4), // .extensions(Extensions::IMPLICIT_SOME),
        )?;

        fs::write(DISK_RON, ron_str)?;

        Ok(())
    }
}
