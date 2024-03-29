# Vhost vsock

## Requirements
- Vhost vsock device support on the Backend VM (e.g. `CONFIG_VHOST_VSOCK` on buildroot)

## Quick start

Follow these steps to quickly set up and run the vhost vsock device with Bao Hypervisor:

1. **Prepare Configuration File**: Create a configuration file (*config-virtio-vsock.yaml*) specifying
the settings for the vhost virtual filesystem device. One example of a configuration file could be:

```
devices:
    # --- Common ---
  - id: 0
    type: "vsock"
    shmem_addr: 0x50000000
    shmem_size: 0x01000000
    shmem_path: "/dev/baoipc0"
    mmio_addr: 0xa003e00
    irq: 47
    data_plane: vhost
    # --- Vsock specific ---
    guest_cid: 3
    # -----------------------------
```

2. Launch the **device model** with vhost vsock frontend device:

```
nohup bao-virtio --config /PATH/TO/YOUR/config-virtio-vsock.yaml > /etc/bao-virtio.log 2>&1 &
```