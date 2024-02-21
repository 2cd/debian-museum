mod builder;
use std::io;

#[test]
fn new_map() -> io::Result<()> {
    let name = "linux_oci_platform";
    let pairs = [
        ("aarch32", "linux/arm/v7"),
        ("aarch64", "linux/arm64"),
        ("alpha", "linux/alpha"),
        ("amd64", "linux/amd64"),
        ("arm", "linux/arm/v6"),
        ("arm64", "linux/arm64"),
        ("armv3", "linux/arm/v3"),
        ("armv4", "linux/arm/v4"),
        ("armv4t", "linux/arm/v4"),
        ("armv5", "linux/arm/v5"),
        ("armv5t", "linux/arm/v5"),
        ("armv5te", "linux/arm/v5"),
        ("armv6", "linux/arm/v6"),
        ("armv7", "linux/arm/v7"),
        ("armv7a", "linux/arm/v7"),
        ("armv7l", "linux/arm/v7"),
        ("armv8m", "linux/arm/v7"),
        ("armv8a", "linux/arm64"),
        ("armv9", "linux/arm64"),
        ("hppa", "linux/hppa"),
        ("i386", "linux/386"),
        ("i486", "linux/386"),
        ("i586", "linux/386"),
        ("i686", "linux/386"),
        ("ia64", "linux/ia64"),
        ("lpia", "linux/386"),
        ("loong64", "linux/loong64"),
        ("loongarch64", "linux/loong64"),
        ("m68k", "linux/m68k"),
        ("mips", "linux/mips"),
        ("mips64", "linux/mips64"),
        ("mips64be", "linux/mips64"),
        ("mips64el", "linux/mips64le"),
        ("mips64le", "linux/mips64le"),
        ("mipsbe", "linux/mips"),
        ("mipsel", "linux/mipsle"),
        ("mipsle", "linux/mipsle"),
        ("powerpc", "linux/ppc"),
        ("powerpc64el", "linux/ppc64le"),
        ("powerpc64le", "linux/ppc64le"),
        ("ppc", "linux/ppc"),
        ("ppc64el", "linux/ppc64le"),
        ("ppc64le", "linux/ppc64le"),
        ("ppc64", "linux/ppc64"),
        ("rv64", "linux/riscv64"),
        ("riscv64", "linux/riscv64"),
        ("riscv64gc", "linux/riscv64"),
        ("rv64gc", "linux/riscv64"),
        ("rv64imafdc", "linux/riscv64"),
        ("s390", "linux/s390"),
        ("s390x", "linux/s390x"),
        ("sh4", "linux/sh4"),
        ("sparc", "linux/sparc"),
        ("sparc64", "linux/sparc64"),
        ("x32", "linux/amd64p32"),
        ("x64", "linux/amd64"),
        ("x86", "linux/386"),
        ("x86_64", "linux/amd64"),
    ];

    builder::MapBuilder::new(name, &pairs).build()
}

#[test]
fn get_map() {
    // dbg!(env::current_dir());
    let a = archmap::linux_oci_platform::map()
        .get("x86")
        .copied()
        .unwrap();

    dbg!(a);
}
