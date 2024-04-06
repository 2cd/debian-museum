# DEV VM

DEV VM is based on MINIMAL VM, with the following differences:

- It comes with a "built-in" kernel and uses GRUB for booting.
- Docker and qemu-user-static are pre-installed.

Before starting the virtual machine, you can expand the disk using `qemu-img`:

```sh
qemu-img resize disk.qcow2 +10G
```

Then run the virtual machine:

```sh
./run
```

Next, reboot the virtual machine using the `reboot` command.

Finally, the `unsafe-resize-partition.service` (systemd) will automatically resize the disk partition size.

If it does not work, please feel free to report an issue.

> Before reporting, please include the log.
> Run `journalctl -u unsafe-resize-partition.service` and get the output.
