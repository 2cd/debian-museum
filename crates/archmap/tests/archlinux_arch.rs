mod builder;
use std::io;

#[test]
fn new_map() -> io::Result<()> {
    let name = "arch_os";
    let pairs = [
        ("aarch32", "armv7h"),
        ("aarch64", "aarch64"),
        ("alpha", "alpha"),
        ("amd64", "x86_64"),
        ("arm", "arm"),
        ("arm64", "aarch64"),
        ("armv3", "arm"),
        ("armv4", "arm"),
        ("armv4t", "arm"),
        ("armv5", "arm"),
        ("armv5t", "arm"),
        ("armv5te", "arm"),
        ("armv6", "armv6h"),
        ("armv6h", "armv6h"),
        ("armv7h", "armv7h"),
        ("armv7", "armv7h"),
        ("armv7a", "armv7h"),
        ("armv7l", "armv7h"),
        ("armv8m", "armv7h"),
        ("armv8a", "aarch64"),
        ("armv9", "aarch64"),
        ("hppa", "hppa"),
        ("i386", "i486"),
        ("i486", "i486"),
        ("i586", "i486"),
        ("i686", "i686"),
        ("ia64", "ia64"),
        ("lpia", "pentium4"),
        ("loong64", "loong64"),
        ("loongarch64", "loong64"),
        ("m68k", "m68k"),
        //
        // ("mips", "mipsle"),
        ("mips", "mips"),
        ("mips64", "mips64le"),
        ("mips64be", "mips64"),
        ("mips64el", "mips64le"),
        ("mips64le", "mips64le"),
        ("mipsbe", "mips"),
        ("mipsel", "mipsle"),
        ("mipsle", "mipsle"),
        ("powerpc", "powerpc"),
        ("powerpc64el", "powerpc64le"),
        ("powerpc64le", "powerpc64le"),
        ("ppc", "powerpc"),
        ("ppc64el", "powerpc64le"),
        ("ppc64le", "powerpc64le"),
        ("powerpc64", "powerpc64"),
        ("ppc64", "powerpc64"),
        ("rv64", "riscv64"),
        ("riscv64", "riscv64"),
        ("riscv64gc", "riscv64"),
        ("rv64gc", "riscv64"),
        ("rv64imafdc", "riscv64"),
        //
        // ("s390", "s390x"),
        ("s390", "s390"),
        ("s390x", "s390x"),
        ("sh4", "sh4"),
        ("sparc", "sparc"),
        ("sparc64", "sparc64"),
        ("x32", "x32"),
        ("amd64p32", "x32"),
        ("x64", "x86_64"),
        ("x64v2", "x86_64"),
        ("x64v3", "x86_64"),
        ("x64v4", "x86_64"),
        ("pentium4", "pentium4"),
        ("x86", "pentium4"),
        ("x86_64", "x86_64"),
        ("x86-64-v2", "x86_64"),
        ("x86-64-v3", "x86_64"),
        ("x86-64-v4", "x86_64"),
    ];

    builder::MapBuilder::new(name, &pairs).build()
}

#[test]
fn get_map() {
    let a = archmap::arch_os::map()
        .get("arm64")
        .copied()
        .unwrap();
    assert_eq!(a, "aarch64");
}
