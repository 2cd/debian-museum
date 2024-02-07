#![cfg(unix)]

mod cfg;
mod cli;
mod command;
mod dir;
mod docker;
mod logger;
mod task;
mod url;

fn main() -> anyhow::Result<()> {
    use clap::Parser;

    logger::init();
    log::debug!("parsing command line args ...");
    cli::Cli::parse().run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::disk::DiskV1;
    use std::{env, fs, path::Path};

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
        use ron::{extensions::Extensions, ser::PrettyConfig};

        let ron_file = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/old_old_debian/disk.v1.ron"
        );

        // logger::init();
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
