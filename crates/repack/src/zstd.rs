use crate::{
    compression::Upack,
    cpu,
    io_buffer::{buf_reader, create_file},
};
use std::{io, path::Path};

impl<S, D> Upack<S, D>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    pub(crate) fn compress_to_zst(&self, level: i32) -> io::Result<()> {
        // let zstd_file = buf_writer(&self.target)?;

        // > The zstd library has its own internal input buffer
        // https://docs.rs/zstd/latest/zstd/stream/write/struct.Encoder.html
        let zstd_file = create_file(&self.target.path)?;

        let mut encoder = zstd::Encoder::new(zstd_file, level)?;
        encoder.multithread(*cpu::num() as _)?;

        let mut src_file = buf_reader(&self.source.path)?;
        io::copy(&mut src_file, &mut encoder)?;
        encoder.finish()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rsplit_3() {
        let s = "a.x.base.tar.zst";
        let mut fmt_arr = [""; 2];
        for (i, value) in s
            .rsplitn(3, '.')
            .take(2)
            .enumerate()
        {
            unsafe { *fmt_arr.get_unchecked_mut(i) = value }
        }
        fmt_arr.reverse();
        // dbg!(fmt_arr);
        assert_eq!(fmt_arr, ["tar", "zst"]);
    }
}
