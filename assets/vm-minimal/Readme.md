# MINIMAL VM

## Step1: install qemu

Alpine

```sh
# If the VM is x64 arch:
apk add qemu-system-x86_64

# If the VM is arm64 arch:
apk add qemu-system-aarch64

# If the VM is rv64 arch:
apk add qemu-system-riscv64

# If the VM is loong64 arch:
apk add qemu-system-loongarch64
```

Android Termux

```sh
# If the VM is x64 arch:
pkg i qemu-system-x86-64-headless

# If the VM is arm64 arch:
pkg i qemu-system-aarch64-headless
```

ArchLinux

```sh
# If the VM is x64 arch:
pkg i qemu-system-x86

# If the VM is arm64 arch:
pkg i qemu-system-aarch64
```

Debian, Ubuntu, Kali, Mint, ...

```sh
# run apt as root (e.g., sudo apt update)
apt update
apt install qemu-system
```

Fedora

```sh
# If the VM is x64 arch:
dnf install qemu-system-x86

# If the VM is arm64 arch:
dnf install qemu-system-aarch64
```

## Step2: fix KVM permissions (Optional)

Add the current user to the kvm user group.

```sh
# run it as root (i.e., +sudo/+sudo-rs/+doas)
usermod -aG kvm "$(id -un)"
```

If it does not work, change he permissions manually.

```sh
# run it as root
chmod 666 -v /dev/shm
```

## Step3: install zsh

On most distributions, the package name for zsh is `zsh`, and you can install it using your system's package manager.

You can also download it from the [2moe/zsh-static-docker](https://github.com/2moe/zsh-static-docker/releases) repository. (Extract tar.zst and get `opt/bin/zsh`, move it to `${XDG_BIN_HOME:~/.local/bin}`)

## Step4: Expand virtual disk (Optional)

Install `qemu-utils`, then run `qemu-img`.

```sh
qemu-img resize disk.img +2G
```

If `unsafe-resize-partition.service` (systemd) works correctly, start the virtual machine "**twice**" (i.e., Start once, then reboot.) and it will automatically resize the partitions to utilize the unallocated space.

## Step5: run

```sh
./run
```

> localhost login: root

## Step6: Other (Optional)

install OpenSSH client

> The ssh package name for debian/ubuntu is `ssh`, or `openssh` in some distributions.

If you don't need sshd, install the ssh client separately.

```sh
# run apt as root (i.e., +sudo/+sudo-rs/+doas)
apt install openssh-client
```

### connect to ssh

Open a new terminal window, then run:

```sh
./connect-to-ssh
```

### send files to VM

Usage:

```sh
./send-file-to-vm [host-file-path-1] [host-file-path-2] ... [host-file-path-100]
```

Example:

```sh
./send-file-to-vm ./file1.txt ./file2.tar.zst
# By default, it will be sent to '/root/Downloads' within the VM.
```

### get files from VM

Usage:

```sh
./get-file-from-vm [vm-file-path-1] [vm-file-path-2] ... [vm-file-path-100]
```

Example 1:

```sh
./get-file-from-vm /etc/os-release
```

Example 2:

```zsh
# zsh
files=(
    /etc/os-release
    /etc/issue
    /etc/apt/sources.list.d/mirror.sources
)

./get-file-from-vm $files

# By default, it will download to "./Downloads"
```

### install docker

Minimal VM uses an "external" kernel.
You need to install an "internal" kernel in the virtual machine, otherwise docker will not work.

Run the following command in the virtual machine.

```sh
apt update
apt install docker.io

# If the VM is x64 arch:
apt install linux-image-cloud-amd64

# If the VM is arm64 arch:
apt install linux-image-cloud-arm64
```

> You can also use a DEV VM with docker pre-installed.