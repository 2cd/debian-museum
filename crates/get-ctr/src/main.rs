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
    // use super::*;
    use crate::cfg::{debootstrap, disk::DiskV1};
    use std::{env, fs, path::Path};

    #[test]
    fn convert_toml_to_ron() -> anyhow::Result<()> {
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
            let new_file = Path::new(file).with_extension("ron");

            let ron_str = match name {
                "debian" | "ubuntu" => {
                    let value = toml::from_str::<debootstrap::Cfg>(
                        &fs::read_to_string(file)?,
                    )?;
                    ron::to_string(&value)
                }
                _ => {
                    let value =
                        toml::from_str::<DiskV1>(&fs::read_to_string(file)?)?;

                    ron::to_string(&value)
                }
            }?;

            fs::write(new_file, ron_str)?;
        }

        Ok(())
    }
}
