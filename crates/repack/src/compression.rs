use core::fmt::Debug;
use std::{io, path::Path};

use crate::format::Format;

/// - Decode
///     - Full: a-dir.tar.zst => a-dir
///     - OuterMost: a-dir.tar.zst => a-dir.tar
#[derive(Debug, Copy, Clone)]
pub enum Layer {
    Full,
    OuterMost,
}

impl Default for Layer {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    Decode(Layer),
    Encode { level: u32 },
}

impl Operation {
    pub fn encode(level: Option<u32>) -> Self {
        Self::Encode {
            level: level.unwrap_or(9),
        }
    }

    /// Decode(OuterMost)
    pub fn decode() -> Self {
        Self::Decode(Layer::OuterMost)
    }

    /// Decode(Full)
    pub fn decode_full() -> Self {
        Self::Decode(Layer::Full)
    }
}

impl Default for Operation {
    fn default() -> Self {
        Self::decode()
    }
}

#[derive(Debug, Default)]
pub struct UpackFile<P>
where
    P: AsRef<Path>,
{
    pub path: P,
    pub(crate) format: Format,
}

impl<P> UpackFile<P>
where
    P: AsRef<Path>,
{
    pub fn new(path: P) -> Self {
        let format = Format::from(to_lowercase_file_name(&path));

        Self { path, format }
    }

    // TODO: Using Magic Bytes instead of FileName
    // pub fn new_src() {}

    pub fn get_format(&self) -> &Format {
        &self.format
    }
}

fn to_lowercase_file_name<P: AsRef<Path>>(p: P) -> Option<String> {
    p.as_ref().file_name().map(|x| {
        x.to_string_lossy()
            .to_ascii_lowercase()
    })
}

#[derive(Debug, Default)]
pub struct Upack<S, D>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    pub source: UpackFile<S>,
    pub target: UpackFile<D>,
    pub operation: Operation,
}

impl<S, D> Upack<S, D>
where
    S: AsRef<Path> + Debug,
    D: AsRef<Path> + Debug,
{
    /// Creates an instance of Upack.
    ///
    /// The default operation is `Decode(Layer::OuterMost)`.
    ///
    /// # Examples
    ///
    /// ## Decompress **base.tgz** to **base.tar**
    ///
    /// ```no_run
    /// let de_gz = Upack::new("base.tgz", "base.tar");
    ///
    /// de_gz.run()?;
    /// ```
    ///
    /// ## Compress **file.tar** to **file.tar.zst**
    ///
    /// ```no_run
    /// let compress_to_zstd = Upack::new("file.tar", "file.tar.zst")
    ///     // .with_operation(Operation::Encode { level: 22 });
    ///     .encode_with_max_lv();
    ///
    /// compress_to_zstd.run()?;
    /// ```
    pub fn new(source: S, target: D) -> Self {
        Self {
            source: UpackFile::new(source),
            target: UpackFile::new(target),
            operation: Default::default(),
        }
    }

    pub fn encode_with_max_lv(mut self) -> Self {
        self.operation = {
            Operation::encode(
                self.target
                    .get_format()
                    .max_level(),
            )
        };
        self
    }

    pub fn with_operation(mut self, operation: Operation) -> Self {
        self.operation = operation;
        self
    }

    /// Decode gz or Encode zst
    pub fn run(&self) -> io::Result<()> {
        let src_fmt = self.source.get_format();
        let dst_fmt = self.target.get_format();
        {
            use Format::*;
            use Layer::OuterMost;
            use Operation::*;

            match (&self.operation, src_fmt, dst_fmt) {
                // (Encode{level}, s, TarZstd) if !s.is_tar() => self.compress_to_tar_zst(*level as _)?,
                (Encode { level }, _, TarZstd | Zstd) => {
                    self.compress_to_zst(*level as _)?
                }
                (Decode(OuterMost), Gz | TarGz, _) => self.decompress_gz()?,
                (op, src, dst) => {
                    panic!(
                        "[FATAL] Unsupported operation: {op:?}\n\
                        {self:?}\n\t\
                        src-fmt:{src:?}\n\t\
                        dst-fmt: {dst:?}
                        "
                    )
                }
            }
        }
        Ok(())
    }
}
