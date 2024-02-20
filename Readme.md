# Build

Note: Debian Trixie (kernel: 6.x, arch: amd64 (x64)) cannot build rootfs for debian-wheezy & ubuntu-oneiric and lower versions of amd64 (x64) with debootstrap.

Running the ubuntu `{warty / hoary}` (arch: i386 (x86))  containers results in the following error:

```sh
Inconsistency detected by ld.so: rtld.c: 1192: dl_main: Assertion `(void *) ph->p_vaddr == _rtld_local._dl_sysinfo_dso' failed!
```

To solve this problem, we need to use an older system (with an older kernel).

Building rootfs for Debian Wheezy amd64(x64) can be done with Debian Stretch amd64(x64), but building Ubuntu Warty i386(x86) (4.10, 2004-10, the first publicly released version of ubuntu) requires a very, very old system.

## Preparations

- runs-on: Debian Stretch (ELTS)
- user: root (uid = 0)
- depends:
  - systemd-container
  - debootstrap
  - curl
- shell: zsh
<!--
- recommends:
  - qemu-user-static 
-->

### add scripts-dir to path

```zsh
# Allows the use of `#` in interactive zsh
setopt interactive_comments

curl -LO "https://github.com/2cd/debian-museum/archive/refs/heads/ci.tar.gz"
tar -xf ci.tar.gz

path=(
    $PWD/debian-museum-ci/src
    $path
)
```

## wheezy-elts-amd64

```zsh
series=wheezy
arch=amd64
name=$series-$arch

url="https://deb.freexian.com/extended-lts/"

args=(
    --include="freexian-archive-keyring,apt-utils,eatmydata"
    # 
    # The command executed in the sub-shell is equivalent to: ["main", "contrib", "non-free"].join(',') => "main,contrib,non-free"
    --components=$(get-deb-components deb-debootstrap)

    --no-check-gpg
    --arch=$arch
    $series
    $name
    $url
)
debootstrap $args

# zsh's slice index starts at 1 (not 0). Setting the 5th character to the empty char is equivalent to replacing `https://` with `http://`
url[5]=''

echo "deb [trusted=yes] $url $series $(get-deb-components deb)" > $name/etc/apt/sources.list

run-apt-upgrade-in-container $name
pack-rootfs-to-tar $name
```

## debian-squeeze-amd64

```zsh
series=squeeze
arch=amd64
name=$series-$arch

mirror_url='https://mirrors.nju.edu.cn/debian-archive'
official_url='http://archive.debian.org'

args=(
    --include="apt-utils,eatmydata"
    --exclude="apt-transport-https"
    #
    --no-check-gpg
    --components=$(get-deb-components deb-debootstrap)
    --arch=$arch
    $series
    $name
    $mirror_url/debian
)
debootstrap $args


update_src_list() {
    local url=$1
    local components=$(get-deb-components deb)

    > $name/etc/apt/sources.list <<-EOF
deb [trusted=yes] $url/debian/ $series-lts $components
deb [trusted=yes] $url/debian/ $series $components
deb [trusted=yes] $url/debian-backports/ $series-backports $components
deb [trusted=yes] $url/debian-security/ $series/updates $components
EOF
}

# https:// -> http://
mirror_url[5]=''
update_src_list $mirror_url

run-apt-upgrade-in-container $name

update_src_list $official_url
test-apt-update-in-container $name

pack-rootfs-to-tar $name
```

## debian-lenny-amd64

```zsh
series=lenny
arch=amd64
name=$series-$arch

pkgs=( debian-backports-keyring )
mirror_url='https://mirrors.nju.edu.cn/debian-archive'
official_url='http://archive.debian.org'

args=(
    --include="apt-utils"
    --exclude="apt-transport-https"
    # 
    --no-check-gpg
    --components=$(get-deb-components deb-debootstrap)
    --arch=$arch
    $series
    $name
    $mirror_url/debian
)
debootstrap $args

update_src_list() {
    local url=$1
    local components=$(get-deb-components deb)

    > $name/etc/apt/sources.list <<-EOF
deb [trusted=yes] $url/debian/ $series $components
deb [trusted=yes] $url/debian-backports/ $series-backports $components
deb [trusted=yes] $url/debian-security/ $series/updates $components

# debian-volatile
# deb [trusted=yes] $url/debian-volatile/ $series/volatile-sloppy $components
# deb [trusted=yes] $url/debian-volatile/ $series/volatile $components
EOF
}

# https:// -> http://
mirror_url[5]=''
update_src_list $mirror_url

run-apt-upgrade-in-container $name

install-pkgs-in-container $name $pkgs

update_src_list $official_url
test-apt-update-in-container $name

pack-rootfs-to-tar $name
```

## debian-etch-amd64

```zsh
series=etch
arch=amd64
name=$series-$arch

pkgs=( debian-backports-keyring )
mirror_url='https://mirrors.nju.edu.cn/debian-archive'
official_url='http://archive.debian.org'

args=(
    --include="apt-utils"
    --exclude="apt-transport-https"
    #
    --no-check-gpg
    --components=$(get-deb-components deb-debootstrap)
    --arch=$arch
    $series
    $name
    $mirror_url/debian
)
debootstrap $args

update_src_list() {
    local url=$1
    local components=$(get-deb-components deb)

    > $name/etc/apt/sources.list <<-EOF
deb [trusted=yes] $url/debian/ $series $components
deb [trusted=yes] $url/debian-backports/ $series-backports $components

# debian-security
#deb [trusted=yes] $url/debian-security/ $series/updates $components

# debian-volatile
# deb [trusted=yes] $url/debian-volatile/ $series/volatile-sloppy $components
# deb [trusted=yes] $url/debian-volatile/ $series/volatile $components

# debian::proposed-updates
# deb [trusted=yes] $url/debian/ $series-proposed-updates $components
EOF
}

update_src_list $official_url
run-apt-upgrade-in-container $name

install-pkgs-in-container $name $pkgs

pack-rootfs-to-tar $name
```

## debian-sarge-amd64

```sh
series=sarge
arch=amd64
name=$series-$arch
mirror_url="https://mirrors.nju.edu.cn/debian-archive/debian-amd64/"
official_url="http://archive.debian.org/debian-amd64/"

args=(
    # --exclude=apt-transport-https
    # 
    --no-check-gpg
    --components=$(get-deb-components deb-debootstrap)
    --arch=$arch
    $series
    $name
    # NOTE: The url path for debian sarge (arch: amd64 (x64)) is "/debian-amd64/", not "/debian/"
    $mirror_url
)
debootstrap $args

echo "deb $official_url $series $(get-deb-components deb)" > $name/etc/apt/sources.list

run-apt-upgrade-in-container $name

pack-rootfs-to-tar $name
```

## ubuntu

<!-- 
4.10 & 5.04

patch

```sh
sed '3a\force_md5' -i.bak /usr/share/debootstrap/scripts/warty
```
-->

```sh
build-old-ubuntu-rootfs
```
