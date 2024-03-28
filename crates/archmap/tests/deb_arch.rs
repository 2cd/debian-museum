mod builder;
use std::io;

#[test]
fn new_map() -> io::Result<()> {
    let name = "debian_arch";
    let pairs = [
        ("aarch32", "armhf"),
        ("aarch64", "arm64"),
        ("alpha", "alpha"),
        ("amd64", "amd64"),
        //
        ("arm", "armel"),
        ("arm64", "arm64"),
        ("armv3", "arm"),
        ("armv4", "armel"),
        ("armv4t", "armel"),
        ("armv5", "armel"),
        ("armv5t", "armel"),
        ("armv5te", "armel"),
        ("armv6h", "armel"),
        ("armv6", "armel"),
        ("armv7", "armhf"),
        ("armv7h", "armhf"),
        ("armv7a", "armhf"),
        ("armv7l", "armhf"),
        ("armv8m", "armhf"),
        ("armv8a", "arm64"),
        ("armv9", "arm64"),
        //
        ("hppa", "hppa"),
        ("i386", "i386"),
        ("i486", "i386"),
        ("i586", "i386"),
        ("i686", "i386"),
        ("ia64", "ia64"),
        ("lpia", "lpia"),
        ("loong64", "loong64"),
        ("loongarch64", "loong64"),
        ("m68k", "m68k"),
        //
        // ("mips", "mipsel"),
        ("mips", "mips"),
        ("mips64", "mips64el"),
        ("mips64be", "mips64"),
        ("mips64el", "mips64el"),
        ("mips64le", "mips64el"),
        ("mipsbe", "mips"),
        ("mipsel", "mipsel"),
        ("mipsle", "mipsel"),
        //
        ("powerpc", "powerpc"),
        ("powerpc64el", "ppc64el"),
        ("powerpc64le", "ppc64el"),
        ("ppc", "powerpc"),
        ("ppc64el", "ppc64el"),
        ("ppc64le", "ppc64el"),
        ("powerpc64", "ppc64"),
        ("ppc64", "ppc64"),
        //
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
        //
        ("x64", "amd64"),
        ("x64v2", "amd64"),
        ("x64v3", "amd64"),
        ("x64v4", "amd64"),
        ("x86", "i386"),
        ("x86_64", "amd64"),
        ("x86-64-v2", "amd64"),
        ("x86-64-v3", "amd64"),
        ("x86-64-v4", "amd64"),
    ];

    builder::MapBuilder::new(name, &pairs).build()
}

#[test]
fn get_map() {
    // dbg!(env::current_dir());
    let a = archmap::debian_arch::map()
        .get("x86")
        .copied()
        .unwrap();
    assert_eq!(a, "i386");
}
