use crate::ByteUnit;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

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
            GiB(v) => serializer.serialize_str(&format!("{v} GiB")),
            TiB(v) => serializer.serialize_str(&format!("{v} TiB")),
        }
    }
}
