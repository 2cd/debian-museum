use core::{num::ParseFloatError, str::FromStr};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseByteUnitError {
    #[error("Failed to parse float from str")]
    ParseFloat(#[from] ParseFloatError),

    #[error("Empty Float String")]
    EmptyFloatStr,

    #[error("Empty Unit String")]
    EmptyUnitStr,
    // #[error(transparent)]
    // Other(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum ByteUnit {
    KiB(f32),
    MiB(f32),
}

impl Default for ByteUnit {
    fn default() -> Self {
        Self::KiB(f32::NAN)
    }
}

impl FromStr for ByteUnit {
    type Err = ParseByteUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split_whitespace();

        let float_str = s
            .next()
            .ok_or(ParseByteUnitError::EmptyFloatStr)?;
        let float = f32::from_str(float_str)?;

        match s.next() {
            Some("KiB") => Ok(Self::KiB(float)),
            Some("MiB") => Ok(Self::MiB(float)),
            _ => Err(ParseByteUnitError::EmptyUnitStr),
        }
    }
}

impl<'de> Deserialize<'de> for ByteUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let new = Self::from_str(&String::deserialize(deserializer)?)
            .map_err(Error::custom)?;
        Ok(new)
    }
}

impl Serialize for ByteUnit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ByteUnit::*;
        match self {
            KiB(v) => serializer.serialize_str(&format!("{v} KiB")),
            MiB(v) => serializer.serialize_str(&format!("{v} MiB")),
        }
    }
}

impl ByteUnit {
    const BYTES_KIB: f32 = 1024.;
    const BYTES_MIB: f32 = 1024. * 1024.;

    /// # Example
    ///
    /// ```no_run
    /// let bytes = 4096;
    /// let kib = ByteUnit::new_kib(bytes);

    /// ```
    pub fn new_kib(bytes: u64) -> Self {
        let kib_100x = (bytes as f32 / Self::BYTES_KIB) * 100.0;
        Self::KiB(kib_100x.round() / 100.0)
    }

    pub fn new_mib(bytes: u64) -> Self {
        let mib_100x = (bytes as f32 / Self::BYTES_MIB) * 100.0;
        Self::MiB(mib_100x.round() / 100.0)
    }
}
