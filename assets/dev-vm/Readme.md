# DEV VM

step1: install deps

```sh
sudo apt install qemu-system-x86 podman virtiofsd
```

step2 (optional):

Add the current user to the kvm user group.

```sh
sudo usermod -aG kvm "$(id -un)"
```

If it does not work, change the permissions on `/dev/kvm` manually.

```sh
sudo chmod 666 -v /dev/shm
```

step3: run

```sh
./run
```
