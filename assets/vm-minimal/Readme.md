# DEV VM

Step1: install qemu

```sh
sudo apt install qemu-system-x86
```

Step2: fix KVM Permissions (Optional):

Add the current user to the kvm user group.

```sh
sudo usermod -aG kvm "$(id -un)"
```

If it does not work, change the permission manually.

```sh
sudo chmod 666 -v /dev/shm
```

Step3: run

```sh
./run
```

> localhost login: root

Step4: connect to ssh (Optional)

Open a new terminal window, then run:

```sh
./connect-to-ssh
```
