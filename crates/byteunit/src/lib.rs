pub mod error;
pub mod serialization;

use crate::error::ParseByteUnitError;
use core::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteUnit {
    KiB(f32),
    MiB(f32),
    GiB(f32),
    TiB(f32),
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
            Some("GiB") => Ok(Self::GiB(float)),
            Some("TiB") => Ok(Self::TiB(float)),
            _ => Err(ParseByteUnitError::EmptyUnitStr),
        }
    }
}

impl ByteUnit {
    const BYTES_KIB: f32 = 1024.;
    const BYTES_MIB: f32 = 1024. * Self::BYTES_KIB;
    const BYTES_GIB: f32 = 1024. * Self::BYTES_MIB;
    const BYTES_TIB: f32 = 1024. * Self::BYTES_GIB;

    /// Creates a new instance of ByteUnit.
    /// Automatically determines the enum type based on byte_size.
    ///
    /// - `2 * 1024` bytes => KiB(2.0)
    /// - `3 * 1024 * 1024` Bytes => MiB(3.0)
    ///
    /// # Example
    ///
    /// ```no_run
    /// let byte_size = 5.2 as u64 * 1024 * 1024;
    /// let unit = ByteUnit::new(byte_size);
    /// assert_eq!(ByteUnit::MiB(5.0), unit);
    ///
    /// let new_unit = ByteUnit::new(1024 * byte_size);
    /// assert!(new_unit.is_gib());
    /// ```
    pub fn new(bytes: u64) -> Self {
        match bytes as f32 {
            f if f < Self::BYTES_MIB => Self::new_kib(bytes),
            f if f < Self::BYTES_GIB => Self::new_mib(bytes),
            f if f < Self::BYTES_TIB => Self::new_gib(bytes),
            _ => Self::new_tib(bytes),
        }
    }

    /// # Example
    ///
    /// ```no_run
    /// let size = 4096;
    /// let unit = ByteUnit::new_kib(size);
    /// assert_eq!(ByteUnit::KiB(4.0), unit);
    /// ```
    pub fn new_kib(bytes: u64) -> Self {
        let kib_100x = (bytes as f32 / Self::BYTES_KIB) * 100.0;
        Self::KiB(kib_100x.round() / 100.0)
    }

    pub fn new_mib(bytes: u64) -> Self {
        let mib_100x = (bytes as f32 / Self::BYTES_MIB) * 100.0;
        Self::MiB(mib_100x.round() / 100.0)
    }

    pub fn new_gib(bytes: u64) -> Self {
        let gib_100x = (bytes as f32 / Self::BYTES_GIB) * 100.0;
        Self::GiB(gib_100x.round() / 100.0)
    }

    pub fn new_tib(bytes: u64) -> Self {
        let tib_100x = (bytes as f32 / Self::BYTES_TIB) * 100.0;
        Self::TiB(tib_100x.round() / 100.0)
    }

    /// Returns `true` if the byte unit is [`KiB`].
    ///
    /// [`KiB`]: ByteUnit::KiB
    pub fn is_kib(&self) -> bool {
        matches!(self, Self::KiB(..))
    }

    /// Returns `true` if the byte unit is [`MiB`].
    ///
    /// [`MiB`]: ByteUnit::MiB
    pub fn is_mib(&self) -> bool {
        matches!(self, Self::MiB(..))
    }

    /// Returns `true` if the byte unit is [`GiB`].
    ///
    /// [`GiB`]: ByteUnit::GiB
    pub fn is_gib(&self) -> bool {
        matches!(self, Self::GiB(..))
    }

    /// Returns `true` if the byte unit is [`TiB`].
    ///
    /// [`TiB`]: ByteUnit::TiB
    pub fn is_tib(&self) -> bool {
        matches!(self, Self::TiB(..))
    }
}

#[cfg(test)]
mod tests {
    use crate::ByteUnit;

    #[test]
    fn new_byteunit() {
        let byte_size = 5.2 as u64 * 1024 * 1024;
        let unit = ByteUnit::new(byte_size);
        assert_eq!(ByteUnit::MiB(5.0), unit);

        let new_unit = ByteUnit::new(1024 * byte_size);
        assert!(new_unit.is_gib());
    }
}
