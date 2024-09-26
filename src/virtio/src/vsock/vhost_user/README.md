# Vhost-user vsock

## Quick start

Follow these steps to quickly set up and run the vhost-user vsock device with Bao Hypervisor.

1. **Prepare Configuration File**: Create a configuration file (*config-virtio-vsock.yaml*) specifying
the settings for the vhost-user vsock device. One example of a configuration file could be:

```
devices:
  - id: 0
    type: "vsock"
    mmio_addr: 0xa003e00
    data_plane: vhost_user
    socket_path: "/tmp/"
```

2. Launch the [vhost-user vsock backend](https://github.com/rust-vmm/vhost-device/tree/main/vhost-device-vsock):
```
nohup vhost-device-vsock --vm guest-cid=4,uds-path=/tmp/vm4.vsock,socket=/tmp/Vsock.sock > /etc/vhost-vsock.log 2>&1 &
```

3. Launch the device model:

```
nohup bao-virtio-dm --config /PATH/TO/YOUR/config-virtio-vsock.yaml > /etc/bao-virtio-dm.log 2>&1 &
```