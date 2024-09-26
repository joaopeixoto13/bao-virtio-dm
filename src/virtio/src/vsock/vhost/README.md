# Vhost vsock

## Requirements
- Vhost vsock device support on the Backend VM (e.g. `CONFIG_VHOST_VSOCK` on buildroot)

## Quick start

Follow these steps to quickly set up and run the vhost vsock device with Bao Hypervisor:

1. **Prepare Configuration File**: Create a configuration file (*config-virtio-vsock.yaml*) specifying
the settings for the vhost net device. One example of a configuration file could be:

```
devices:
    # --- Common ---
  - id: 0
    type: "vsock"
    mmio_addr: 0xa003e00
    data_plane: vhost
    # --- Vsock specific ---
    guest_cid: 3
    # -----------------------------
```

2. Launch the **device model** with vhost vsock frontend device:

```
nohup bao-virtio-dm --config /PATH/TO/YOUR/config-virtio-vsock.yaml > /etc/bao-virtio-dm.log 2>&1 &
```