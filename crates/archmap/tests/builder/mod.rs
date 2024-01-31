use std::{fs::File, io, io::Write};

#[derive(Debug)]
pub(crate) struct MapBuilder<'a> {
    pub(crate) mod_name: &'a str,
    pub(crate) pairs: &'a [(&'a str, &'a str)],
}

impl<'a> MapBuilder<'a> {
    pub(crate) fn new(mod_name: &'a str, pairs: &'a [(&str, &str)]) -> Self {
        Self { mod_name, pairs }
    }

    pub(crate) fn build(self) -> io::Result<()> {
        let Self { mod_name, pairs } = self;

        let mut map = phf_codegen::Map::new();

        for (key, value) in pairs {
            map.entry(key, &format!(r######"r###"{}"###"######, value));
        }

        let mut file =
            io::BufWriter::new(File::create(format!("src/{}.rs", mod_name))?);

        println!("pub mod {};", mod_name);

        writeln!(
            &mut file,
            "pub const fn map() -> crate::PhfMap {{\n    \
            {}\n\
        }}",
            map.build()
        )
    }
}
