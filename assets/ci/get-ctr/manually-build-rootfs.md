# Build

host: Debian Stretch (ELTS)
depends:
    - systemd-container
    - debootstrap
    - qemu-user-static

## wheezy-elts-amd64

build rootfs

```sh
os=wheezy
arch=amd64
name=$os-$arch

debootstrap \
    --include=freexian-archive-keyring,apt-utils,eatmydata \
    --no-check-gpg \
    --components=main,contrib,non-free \
    --arch=$arch \
    $os \
    $name \
    https://deb.freexian.com/extended-lts/

echo 'deb [trusted=yes] http://deb.freexian.com/extended-lts/ wheezy main contrib non-free' > $name/etc/apt/sources.list

systemd-nspawn -D $name sh -c '
    apt-get update
    apt-get dist-upgrade --assume-yes --force-yes
    apt-get clean
    exit'
```

pack to tar

```sh
umount -lvf $name/sys

tar --posix \
    -C $name \
    "--exclude=proc/*" \
    "--exclude=sys/*" \
    "--exclude=tmp/*" \
    "--exclude=var/tmp/*" \
    "--exclude=run/*" \
    "--exclude=mnt/*" \
    "--exclude=media/*" \
    "--exclude=var/cache/apt/pkgcache.bin" \
    "--exclude=var/cache/apt/srcpkgcache.bin" \
    "--exclude=var/cache/apt/archives/*deb" \
    "--exclude=var/cache/apt/archives/partial/*" \
    "--exclude=dev/*" \
    -cf /mnt/$name.tar \
    .  \
    && rm -rf $name
```

## debian-squeeze

```sh
os=squeeze
arch=amd64
name=$os-$arch

debootstrap \
    --exclude=apt-transport-https \
    --include=apt-utils,eatmydata \
    --no-check-gpg \
    --components=main,contrib,non-free \
    --arch=$arch \
    $os \
    $name \
    https://mirrors.nju.edu.cn/debian-archive/debian/

echo 'deb [trusted=yes] http://archive.debian.org/debian squeeze-lts main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian squeeze main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian-backports/ squeeze-backports main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian-security/ squeeze/updates main contrib non-free' > $name/etc/apt/sources.list
systemd-nspawn -D $name sh -c '
    apt-get update
    apt-get dist-upgrade --assume-yes --force-yes
    apt-get clean
    exit'
```

## debian-lenny-amd64

```sh
os=lenny
arch=amd64
name=$os-$arch

debootstrap \
    --exclude=apt-transport-https \
    --no-check-gpg \
    --components=main,contrib,non-free \
    --arch=$arch \
    $os \
    $name \
    https://mirrors.nju.edu.cn/debian-archive/debian/

echo 'deb [trusted=yes] http://archive.debian.org/debian lenny main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian-backports/ lenny-backports main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian-security/ lenny/updates main contrib non-free' > $name/etc/apt/sources.list
systemd-nspawn -D $name sh -c '
    apt-get update
    for i in --include debian-backports-keyring apt-utils eatmydata; do
        apt-get install --assume-yes --force-yes $i
    done
    apt-get dist-upgrade --assume-yes --force-yes
    apt-get clean
    exit'

#unlink $name/root/.bash_history
```

## debian-etch-amd64

```sh
os=etch
arch=amd64
name=$os-$arch

debootstrap \
    --exclude=apt-transport-https \
    --no-check-gpg \
    --components=main,contrib,non-free \
    --arch=$arch \
    $os \
    $name \
    https://mirrors.nju.edu.cn/debian-archive/debian/

# disable etch/updates
echo 'deb [trusted=yes] http://archive.debian.org/debian etch main contrib non-free
deb [trusted=yes] http://archive.debian.org/debian-backports/ etch-backports main contrib non-free
# deb [trusted=yes] http://archive.debian.org/debian-security/ etch/updates main contrib non-free' > $name/etc/apt/sources.list

# +lenny.systemd-nspawn.cmd (+backports-keyring)
```

## debian-sarge-amd64

```sh
os=sarge
arch=amd64
name=$os-$arch

debootstrap \
    --no-check-gpg \
    --components=main,contrib,non-free \
    --arch=$arch \
    $os \
    $name \
    https://mirrors.nju.edu.cn/debian-archive/debian-amd64/

# /debian/ -> /debian-amd64/
echo 'deb http://archive.debian.org/debian-amd64/ sarge main contrib non-free' > $name/etc/apt/sources.list
```

<!-- 
TODO: +old ubuntu

## ubuntu-4.10 & 5.04

DEBIAN_FRONTEND=noninteractive 

```sh
os=warty

os=hoary

https://mirrors.nju.edu.cn/ubuntu-old-releases/ubuntu/
```

```sh
cat > /etc/apt/sources.list <<-'EOF'
deb https://mirror.nju.edu.cn/ubuntu-old-releases/ubuntu/ warty main restricted universe multiverse

deb https://mirror.nju.edu.cn/ubuntu-old-releases/ubuntu/ warty-updates main restricted universe multiverse

deb https://mirror.nju.edu.cn/ubuntu-old-releases/ubuntu/ warty-backports main restricted universe multiverse

deb https://mirror.nju.edu.cn/ubuntu-old-releases/ubuntu/ warty-security main restricted universe multiverse
EOF
```
-->
