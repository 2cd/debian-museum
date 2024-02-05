mod compression;
mod docker;
mod file;
pub(crate) mod old_old_debian;
pub(crate) mod pool;

#[cfg(test)]
mod tests {
    use hash_digest::{blake3, sha256};
    use serde::{Deserialize, Serialize};
    use std::io;

    #[test]
    fn sha256() -> io::Result<()> {
        let file = "Cargo.toml";
        let hash = sha256::get(file)?;
        println!("{hash}");

        Ok(())
    }

    #[test]
    fn blake3() -> io::Result<()> {
        let file = "Cargo.toml";
        let hash = blake3::get(file)?;
        println!("{hash}");

        Ok(())
    }

    #[test]
    fn serde_datetime() -> anyhow::Result<()> {
        // const DESC: &[format_description::FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]:[offset_minute]");

        #[derive(Serialize, Deserialize, Debug)]
        struct Time {
            // #[serde(with = "time::serde::rfc3339")]
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
