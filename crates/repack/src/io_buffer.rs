use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter},
    path::Path,
};

pub fn buf_writer<P: AsRef<Path>>(p: P) -> io::Result<BufWriter<File>> {
    Ok(BufWriter::new(create_file(p)?))
}

pub fn create_file<P: AsRef<Path>>(p: P) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(p)
}

pub fn buf_reader<P: AsRef<Path>>(p: P) -> io::Result<BufReader<File>> {
    Ok(BufReader::new(File::open(p)?))
}
