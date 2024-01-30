use crate::{
    compression::Upack,
    io_buffer::{buf_reader, buf_writer},
};
use flate2::read::MultiGzDecoder;
use std::{
    io::{self, Write},
    path::Path,
};

impl<S, D> Upack<S, D>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    pub(crate) fn decompress_gz(&self) -> io::Result<()> {
        let gz_file = buf_reader(&self.source.path)?;
        let mut decoder = MultiGzDecoder::new(gz_file);

        let mut dst_file = buf_writer(&self.target.path)?;
        io::copy(&mut decoder, &mut dst_file)?;
        dst_file.flush()
    }
}
