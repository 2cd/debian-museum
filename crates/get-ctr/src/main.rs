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
        use ron::{
            extensions::Extensions,
            ser::{to_string_pretty, PrettyConfig},
        };

        let workdir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        if !workdir.exists() {
            return Ok(());
        }
        env::set_current_dir(&workdir)?;

        for (name, file) in [
            ("old", "old_old_debian/disk.v1.toml"),
            ("debian", "debootstrap/debian.toml"),
            ("ubuntu", "debootstrap/ubuntu.toml"),
        ] {
            let pretty = PrettyConfig::default()
                .enumerate_arrays(true)
                .extensions(Extensions::IMPLICIT_SOME)
                .depth_limit(4);

            let ron_file = Path::new(file).with_extension("ron");

            let ron_str = match name {
                "debian" | "ubuntu" => {
                    let value = toml::from_str::<cfg::debootstrap::DebootstrapCfg>(
                        &fs::read_to_string(file)?,
                    )?;
                    to_string_pretty(&value, pretty)?
                }
                _ => {
                    let value =
                        toml::from_str::<DiskV1>(&fs::read_to_string(file)?)?;
                    to_string_pretty(&value, pretty)?
                }
            };

            fs::write(ron_file, ron_str)?;
        }

        Ok(())
    }
}
