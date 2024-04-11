# syntax=docker/dockerfile:1
ARG RUNNER_ARCH
FROM ghcr.io/2cd/pacman-static:${RUNNER_ARCH}
ARG OPT=/opt/qemu
WORKDIR ${OPT}
ARG SCRIPT=unpack_tzst_and_run_qemu
COPY --link --chmod=755 <<'CREATE_SCRIPT' ${SCRIPT}
#!/opt/bin/zsh
set -fexo pipefail
# ----------------
src_dir=${0:a:h}
info_file=/opt/info/qemu_installed
if [[ -e $info_file ]] {
    cd $src_dir
    ./run
    exit $?
}

pacman -Syy --noconfirm tar zstd
files=(vm-*_*.tar.zst)
file=$files[1]
suffix=${file[(ws^_^)-1]}
tmm_arch=${suffix[(ws^.^)1]}
tar -xvf $file
cat Readme.md

local -A qemu_arch_map=(
    x64             x86
    x86             x86
    i386            x86
    i486            x86
    i586            x86
    i686            x86
    pentium4        x86
    rv64gc          riscv
    rv64            riscv
    ppc64le         ppc
    s390x           s390x
    arm64           aarch64
    armv7a          arm
    mipsle          mips
    mipsbe          mips
    mips64le        mips
    loong64         loongarch64
)

qemu_pkg_arch=$qemu_arch_map[$tmm_arch]
if ((! $#qemu_pkg_arch)) {
    qemu_pkg_arch=$tmm_arch
}

pacman -S --noconfirm qemu-img ||:
pacman -S --noconfirm qemu-system-${qemu_pkg_arch} && {
    tmp=$(mktemp)
    print "$qemu_pkg_arch"> $tmp
    install -Dm644 $tmp $info_file
    ./run
}
CREATE_SCRIPT

COPY vm*.tar.zst ${OPT}

LABEL org.opencontainers.image.description "VM for testing only, note: KVM requires `--device=/dev/kvm`"
CMD ["/opt/qemu/unpack_tzst_and_run_qemu"]
