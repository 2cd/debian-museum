# Debian Museum

![debian-museum](./assets/img/debian-museum.jpg)

- [Releases](https://github.com/2cd/debian-museum/releases)
- [Containers](https://github.com/2cd/debian-museum/pkgs/container/debian)

## History

| version     | release           |
| ----------- | ----------------- |
| 0.01 ~ 0.90 | 1993-08 ~ 1993-12 |
| 0.91        | 1994-01           |
| 0.93R5      | 1995-03           |
| 0.93R6      | 1995-10-26        |

---

| version | codename | created    | release    | eol        |
| ------- | -------- | ---------- | ---------- | ---------- |
| 1.1     | Buzz     | 1993-08-16 | 1996-06-17 | 1997-06-05 |
| 1.2     | Rex      | 1996-06-17 | 1996-12-12 | 1998-06-05 |
| 1.3     | Bo       | 1996-12-12 | 1997-06-05 | 1999-03-09 |
| 2.0     | Hamm     | 1997-06-05 | 1998-07-24 | 2000-03-09 |
| 2.1     | Slink    | 1998-07-24 | 1999-03-09 | 2000-10-30 |
| 2.2     | Potato   | 1999-03-09 | 2000-08-15 | 2003-06-30 |
| 3.0     | Woody    | 2000-08-15 | 2002-07-19 | 2006-06-30 |
| 3.1     | Sarge    | 2002-07-19 | 2005-06-06 | 2008-03-31 |
| 4.0     | Etch     | 2005-06-06 | 2007-04-08 | 2010-02-15 |
| 5.0     | Lenny    | 2007-04-08 | 2009-02-14 | 2012-02-06 |

---

| version | codename | created    | release    | eol        | eol-lts    | eol-elts   |
| ------- | -------- | ---------- | ---------- | ---------- | ---------- | ---------- |
| 6.0     | Squeeze  | 2009-02-14 | 2011-02-06 | 2014-05-31 | 2016-02-29 |            |
| 7       | Wheezy   | 2011-02-06 | 2013-05-04 | 2016-04-25 | 2018-05-31 | 2020-06-30 |
| 8       | Jessie   | 2013-05-04 | 2015-04-26 | 2018-06-17 | 2020-06-30 | 2025-06-30 |
| 9       | Stretch  | 2015-04-26 | 2017-06-17 | 2020-07-18 | 2022-06-30 | 2027-06-30 |
| 10      | Buster   | 2017-06-17 | 2019-07-06 | 2022-09-10 | 2024-06-30 | 2029-06-30 |
| 11      | Bullseye | 2019-07-06 | 2021-08-14 | 2024-08-14 | 2026-08-31 | 2031-06-30 |
| 12      | Bookworm | 2021-08-14 | 2023-06-10 | 2026-06-10 | 2028-06-30 | 2033-06-30 |

---

| version | codename | created    | release |
| ------- | -------- | ---------- | ------- |
| 13      | Trixie   | 2023-06-10 | 2025?   |
| 14      | Forky    | 2025-08?   | 2027?   |

---

| version      | codename |
| ------------ | -------- |
| unstable     | Sid      |
| experimental | RC-Buggy |

> Unlike the Debian Releases unstable and testing, [experimental](https://wiki.debian.org/DebianExperimental) isn't a complete distribution, it can work only as an extension of unstable.
>

To enable experimental source, run a sid container and then edit the **/etc/apt/sources.list.d/mirror.sources** file in the container.

<img alt="experimental source" src="./assets/img/experimental.jpg" />

---

> See also:
>
> - [Ubuntu Museum](https://github.com/2cd/ubuntu-museum/)
> - [Debian Project History](https://www.debian.org/doc/manuals/project-history/releases.en.html)
> - [distro-info-data/debian.csv](https://debian.pages.debian.net/distro-info-data/debian.csv)
> - [Wiki/DebianTesting](https://wiki.debian.org/DebianTesting)

## docker

- RUN IT ON zsh.
  - bash is NOT SUPPORTED, NOR is dash
- Just change the values of `ver` & `arch`
- What follows may seem complicated, but it's actually quite simple to follow step-by-step.
- Depends:
  - `docker.io` | `docker`
  - `zsh` (>= 5)
- Recommends:
  - [qemu-user-static](https://tracker.debian.org/pkg/qemu) | [qemu-user-static-binfmt](https://archlinux.org/packages/extra/x86_64/qemu-user-static-binfmt/)

```zsh
setopt interactive_comments

# On modern Linux distro, running very very old debian containers can be problematic! (e.g., host: 12(bookworm, kernel: 6.x), container: 3.1(sarge), host-arch: amd64, container-arch: amd64)
#
# versions: 8, 9, 10, 11, 12, 13, sid
ver=sid

# architectures: "", "riscv64", "rv64", "amd64", "x86_64", "x64", "arm64", "aarch64", "i386", "loong64", "armhf"
# The architectures supported by different versions are not exactly the same.
# For example, trixie(13) no longer supports mipsle (a.k.a., MIPS 32-bit Little Endian),
# sid(unstable) supports more architectures than stable.
arch=""

# --------------------------
dbg() {
    print >&2 -Pr "%F{blue}[DEBUG]%f $*"
}
# if ver.is_empty()
if ((! #ver)) {
    ver=sid
}

# Debian sid supports a very large number of architectures, not all of which are listed here.
rv64=riscv64
x86=386
x64=amd64
aa64=arm64
local -A oci_arch_map=(
    # riscv64imafdc
    rv64gc       $rv64
    riscv64      $rv64
    rv64         $rv64
    #
    x64          $x64
    amd64        $x64
    x86_64       $x64
    #
    aarch64      $aa64
    arm64        $aa64
    #
    loong64      loong64
    loongarch64  loong64
    # i386 ~ i686
    x86          $x86
    i386         $x86
    i486         $x86
    i586         $x86
    i686         $x86
    #
    armhf        armv7
    # In fact, just relying on `uname -m` cannot determine `feature = "+vfp3"` (armhf).
    # armv7l       $armv7
)

# On alpine, if dpkg is installed, it will output musl-linux-[xxx] (e.g., musl-linux-riscv64), not [xxx] (e.g., riscv64).
# For busybox, the dpkg it contains is a lite version. Therefore, the value of `deb_arch` may be empty.
# If `deb_arch.is_empty()`, use `uname`, not `dpkg`.
get_dpkg_architecture() {
    local dpkg_arch=$(dpkg --print-architecture)

    # arch_arr.split('-').last()
    local deb_arch=$dpkg_arch[(ws^-^)-1]

    case $deb_arch {
        ("") uname -m        ;;
        (*)  print $deb_arch ;;
    }
}

# if arch.is_empty()
if ((! #arch)) {
    # arch = if "dpkg".cmd_exists() { dpkg --print-architecture } else { uname -m }
    arch=$(
        if (($+commands[dpkg])) {
            get_dpkg_architecture
        } else {
            uname -m
        }
    )
}

args=(
    # Run a new container
    run

    # Pull image before running ("always"|"missing"|"never") (default "missing")
    # --pull  always

    # Automatically remove the container when it exits
    --rm

    # ( -i ) Keep STDIN open even if not attached
    --interactive

    # ( -t ) Allocate a pseudo-TTY
    --tty

    # Set environment variables
    --env
    # LANG=?, e.g., C.UTF-8, en_US.UTF-8
    LANG=$LANG
)

# Set timezone env.
#
# if "timedatectl".cmd_exists()
if (($+commands[timedatectl])) {
    # args.push(docker_tz_env)
    args+=(
        --env
        # TZ=?, e.g., UTC, Asia/[CITY], Europe/[CITY]
        TZ=$(timedatectl show --property=Timezone --value)
    )
}

# map: oci_arch_map
#   key: arch
#   value => platform
platform=$oci_arch_map[$arch]

# if platform.is_not_empty()
if ((#platform)) {
    args+=(
        # If you want to run containers from other architectures
        # (e.g., host: arm64, container: riscv64),
        # you need to install `qemu-user-static`.
        --platform  linux/$platform
    )
}

case $ver {
    (sid|unstable) is_sid=true  ;;
    (*)            is_sid=false ;;
}

is_loongarch=false
case $platform {
    # Due to the fact that older versions of Debian (such as Buster)
    # do not support RISC-V 64-bit architecture, it is defined as "sid" here.
    # However, this is not accurate.
    # A more reasonable approach would be to create a `HashMap<version_name, arch_set>`
    # that corresponds to different versions and architectures, and then
    # make the determination based on that.
    (*/riscv64) is_sid=true ;;
    (*/loong64)
        is_sid=true
        is_loongarch=true
    ;;
}

repo="ghcr.io/2cd/debian"

if {$is_sid} {
    repo+=-sid
}
if {$is_loongarch} {
    repo+=:loong64
}
dbg repo: $repo

args+=$repo
dbg args: $args

docker $args
```

## TODO

- +Debian GNU/Hurd VM (a.k.a., Virtual Machine) image
- +Debian kFreeBSD VM image

- +ia64 (a.k.a., Intel Itanium architecture) stage1 container image
  - Since modern qemu does not support emulation of the ia64, there is only stage1 and no full rootfs.
