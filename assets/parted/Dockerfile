# syntax=docker/dockerfile:1

ARG BUILDER
ARG TMM_ARCH

FROM --platform=${BUILDPLATFORM} ghcr.io/2moe/zsh-static as zsh
FROM ghcr.io/2moe/zsh-static:${TMM_ARCH} as zsh_target
# ----------
FROM --platform=${BUILDPLATFORM} alpine:edge as patcher
RUN apk add patchelf
# ----------
FROM --platform=${TARGETPLATFORM} ${BUILDER} as builder
COPY --link --from=zsh /opt/bin/zsh /bin
WORKDIR /app/bin

RUN <<'INSTALL_PARTED'
#!/bin/zsh -fex
mkdir -p ../{lib,share}
pkg_mgr=apk
if (($+commands[apt-get])) {
    apt-get update
    pkg_mgr=dpkg
}
apt_i() {
    apt-get install -y $*
}
for i (parted partprobe sfdisk) {
    if [[ $i != partprobe ]] {
        case $pkg_mgr {
            (dpkg) apt_i $i || apt_i fdisk ;;
            (apk)  apk add $i            ;;
        }
        hash -r
    }
    bin=$commands[$i]
    cp -L $bin .

    local -A map=($(
        ldd $i | awk '{
            printf("%s\n%s\n", $1, $(NF - 1))
        }')
    )
    local -p i map

    for src dst (${(kv)map}) {
        case ${dst:t} {
            (linux-vdso.so.*) continue ;;
        }

        cp -fL $dst ../lib/ ||:

        lib_file=../lib/${src:t}

        if [[ ! -e $lib_file ]] {
            ln -svf ${dst:t} $lib_file
        }
    }
    files=(../lib/ld-*-*.so.*)
    print -R ${files[1]:t} > ../share/interpreter.txt
}
INSTALL_PARTED

FROM patcher as patched
COPY --link --from=zsh /opt/bin/zsh /bin
COPY --from=builder /app /app
WORKDIR /app/bin
RUN <<'PATCH_ELF'
#!/bin/zsh -fex
interpreter=$(< ../share/interpreter.txt)
vendor_dir=/var/lib/tmm/vendor
for i (parted partprobe sfdisk) {
    patchelf --set-interpreter ${vendor_dir}/lib/$interpreter $i
    patchelf --set-rpath ${vendor_dir}/lib:.:../lib $i ||:
    tmpfile=$(mktemp)
    > $tmpfile <<'XBIN'
#!/bin/sh
env LD_PRELOAD="" LD_LIBRARY_PATH="_VENDOR_/lib" "_VENDOR_/bin/_BIN_" "$@"
XBIN
    sed -e "s@_VENDOR_@$vendor_dir@g" -e "s@_BIN_@$i@" -i $tmpfile
    install -Dm755 $tmpfile ../xbin/$i
    unlink $tmpfile
}
PATCH_ELF

# FROM patcher
FROM zsh_target
ARG VENDOR=/var/lib/tmm/vendor
COPY --from=patched /app ${VENDOR}
WORKDIR /bin
RUN ln -svf /opt/bin/busybox /bin/sh
WORKDIR ${VENDOR}/xbin
