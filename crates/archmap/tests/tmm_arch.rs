mod builder;
use std::io;

#[test]
fn new_map_v0() -> io::Result<()> {
    let name = "tmm_arch_v0";
    let pairs = [
        ("aarch32", "armv7a"),
        ("aarch64", "arm64"),
        ("alpha", "alpha"),
        ("amd64", "x64"),
        ("arm", "armv5te"),
        ("arm64", "arm64"),
        ("armv3", "armv3"),
        ("armv4", "armv4t"),
        ("armv4t", "armv4t"),
        ("armv5", "armv5te"),
        ("armv5t", "armv5te"),
        ("armv5te", "armv5te"),
        ("armv6", "armv6h"),
        ("armv6h", "armv6h"),
        ("armv7h", "armv7a"),
        ("armv7", "armv7a"),
        ("armv7a", "armv7a"),
        ("armv7l", "armv7a"),
        ("armv8m", "armv7a"),
        ("armv8a", "arm64"),
        ("armv9", "arm64"),
        ("hppa", "hppa"),
        ("i386", "x86"),
        ("i486", "x86"),
        ("i586", "x86"),
        ("i686", "x86"),
        ("ia64", "ia64"),
        ("lpia", "x86"),
        ("loong64", "loong64"),
        ("loongarch64", "loong64"),
        ("m68k", "m68k"),
        //
        // ("mips", "mips"),
        ("mips", "mipsle"),
        ("mips64", "mips64le"),
        ("mips64be", "mips64"),
        ("mips64el", "mips64le"),
        ("mips64le", "mips64le"),
        ("mipsbe", "mipsbe"),
        ("mipsel", "mipsle"),
        ("mipsle", "mipsle"),
        ("powerpc", "ppc"),
        ("powerpc64el", "ppc64le"),
        ("powerpc64le", "ppc64le"),
        ("ppc", "ppc"),
        ("ppc64el", "ppc64le"),
        ("ppc64le", "ppc64le"),
        ("powerpc64", "ppc64"),
        ("ppc64", "ppc64"),
        ("rv64", "rv64gc"),
        ("riscv64", "rv64gc"),
        ("riscv64gc", "rv64gc"),
        ("rv64gc", "rv64gc"),
        ("rv64imafdc", "rv64gc"),
        //
        // ("s390", "s390"),
        ("s390", "s390x"),
        ("s390x", "s390x"),
        ("sh4", "sh4"),
        ("sparc", "sparc"),
        ("sparc64", "sparc64"),
        ("x32", "x32"),
        ("amd64p32", "x32"),
        ("x64", "x64"),
        ("x64v2", "x64v2"),
        ("x64v3", "x64v3"),
        ("x64v4", "x64v4"),
        ("pentium4", "x86"),
        ("x86", "x86"),
        ("x86_64", "x64"),
        ("x86-64-v2", "x64v2"),
        ("x86-64-v3", "x64v3"),
        ("x86-64-v4", "x64v4"),
    ];

    builder::MapBuilder::new(name, &pairs).build()
}

#[test]
fn get_map() {
    let a = archmap::tmm_arch_v0::map()
        .get("aarch64")
        .copied()
        .unwrap();
    assert_eq!(a, "arm64");
}
