pub(crate) mod build_rootfs;
mod compression;
mod docker;
pub(crate) mod old_old_debian;
pub(crate) mod pool;

#[cfg(test)]
mod tests {
    use hash_digest::{blake3, sha256};
    use serde::{Deserialize, Serialize};
    use std::{io, path::Path};

    #[test]
    fn sha256() -> io::Result<()> {
        let file = Path::new("Cargo.toml");
        if !file.exists() {
            return Ok(());
        }

        let hash = sha256::get(file)?;
        println!("{hash}");

        Ok(())
    }

    #[test]
    fn blake3() -> io::Result<()> {
        let file = Path::new("Cargo.toml");
        if !file.exists() {
            return Ok(());
        }

        let hash = blake3::get(file)?;
        println!("{hash}");

        Ok(())
    }

    #[test]
    fn serde_datetime() -> anyhow::Result<()> {
        #[derive(Serialize, Deserialize, Debug)]
        struct Time {
            datetime: time::OffsetDateTime,
        }

        let now = Time {
            datetime: time::OffsetDateTime::now_utc(),
        };

        let ron_str = ron::to_string(&now)?;
        println!("{ron_str}");
        Ok(())
    }
}
