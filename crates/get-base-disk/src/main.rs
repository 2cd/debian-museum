mod cfg;
mod command;
mod docker;
mod logger;
mod task;
mod url;

use log::info;
use std::{
    env, io,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    logger::init();

    let workdir = set_workdir()?;
    info!("Working directory: {:?}", workdir);

    let disk_cfg = cfg::DiskV1::deser()?;
    task::parse_cfg(&disk_cfg, &workdir)?;

    Ok(())
}

fn set_workdir() -> io::Result<PathBuf> {
    let pwd = env::current_dir()?;

    #[cfg(debug_assertions)]
    {
        if pwd.ends_with(Path::new("crates").join(log_l10n::get_pkg_name!())) {
            log::info!("set current dir to ../..");
            env::set_current_dir("../..")?;
            return env::current_dir();
        }
    }
    Ok(pwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::DiskV1;
    use std::fs;

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
        use ron::{extensions::Extensions, ser::PrettyConfig};

        let ron_file = DiskV1::DISK_RON;
        // logger::init();
        set_workdir()?;
        dbg!(env::current_dir());
        let toml_path = Path::new(ron_file).with_extension("toml");
        dbg!(&toml_path);

        let disk_v1 = toml::from_str::<DiskV1>(&fs::read_to_string(toml_path)?)?;

        let pretty = PrettyConfig::default()
            .enumerate_arrays(true)
            // .extensions(Extensions::IMPLICIT_SOME),
            .depth_limit(4);

        let ron_str = ron::ser::to_string_pretty(&disk_v1, pretty)?;
        fs::write(ron_file, ron_str)?;

        Ok(())
    }
}
