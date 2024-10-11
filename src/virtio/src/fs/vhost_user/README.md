# Vhost-user virtual filesystem

To provide Virtual Filesystem functionality, any vhost-user filesystem backend can be utilized. Nonetheless, for the purposes of demonstration, the [vhost-user virtio-fs device backend written in Rust](https://gitlab.com/virtio-fs/virtiofsd) was used.


## Quick start

Follow these steps to quickly set up and run the Vhost-user virtual filesystem device with Bao Hypervisor:

1. **Prepare Configuration File**: Create a configuration file (e.g. *config-virtio-fs.yaml*) specifying
the settings for the vhost-user virtual filesystem device. One example of a configuration file could be:

```
devices:
    # --- VirtIO Common ---
  - id: 0
    type: "fs"
    mmio_addr: 0xa003e00
    data_plane: vhost_user
    # --- Vhost-user specific ---
    socket_path: "/root/"
    # -----------------------------
```

2. Launch your **standalone vhost-user** virtual filesystem device:
```
nohup virtiofsd --socket-path=/root/Fs0.sock --shared-dir /mnt --tag=myfs --announce-submounts --sandbox chroot > /etc/vhost-device-virtiofsd.log 2>&1 &
```

3. Launch the **device model** with the vhost-user virtual filesystem frontend device:

```
nohup bao-virtio-dm --config /PATH/TO/YOUR/config-virtio-fs.yaml > /etc/bao-virtio-dm.log 2>&1 &
```