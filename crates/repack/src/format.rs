#[derive(Debug, Copy, Clone)]
pub enum Format {
    TarZstd,
    Zstd,
    TarGz,
    Gz,
    Tar,
    Unknown,
}

impl Format {
    pub fn max_level(&self) -> Option<u32> {
        use Format::*;
        match self {
            Zstd | TarZstd => Some(22),
            TarGz | Gz => Some(flate2::Compression::best().level()),
            _ => None,
        }
    }

    /// Returns `true` if the format is [`Tar`].
    ///
    /// [`Tar`]: Format::Tar
    pub fn is_tar(&self) -> bool {
        matches!(self, Self::Tar)
    }
}

impl<T: AsRef<str>> From<Option<T>> for Format {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(s) => Self::from_lowercase_str(s.as_ref()),
            _ => Self::Unknown,
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Format {
    /// Splits lowercase filename with `.` and get Format
    ///
    /// - a.zstd => Format::Zstd
    /// - a.tar.zst => Format::TarZstd
    /// - x.y.z.tar.zstd => Format::TarZstd
    /// - f1.pax.gz => Format::TarGz
    /// - f2.tar.gz => Format::TarGz
    /// - f3.tgz  => Format::TarGz
    /// - f4.gzip => Format::Gz
    ///
    /// Note:
    /// - For files that already exist, it is more accurate to recognize the Magic bytes of the file.
    /// - For non-existing files, it is reasonable to determine this by the filename.
    pub fn from_lowercase_str(file_name: &str) -> Self {
        let mut fmt_arr = [""; 2];
        for (i, value) in file_name
            .rsplitn(3, '.')
            .take(2)
            .enumerate()
        {
            unsafe { *fmt_arr.get_unchecked_mut(i) = value }
        }
        fmt_arr.reverse();

        match fmt_arr {
            ["tar" | "pax", "zst" | "zstd"] | [_, "tzst" | "tzstd"] => Self::TarZstd,
            ["tar" | "pax", "gz" | "gzip"] | [_, "tgz" | "tgzip"] => Self::TarGz,
            [_, "zst" | "zstd"] => Self::Zstd,
            [_, "gz" | "gzip"] => Self::Gz,
            [_, "tar" | "pax"] => Self::Tar,
            _ => Self::Unknown,
        }
    }
}
