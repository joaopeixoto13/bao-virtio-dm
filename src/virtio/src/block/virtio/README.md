# VirtIO Block

## Quick start

Follow these steps to quickly set up and run the VirtIO Block device with Bao Hypervisor:

1. **Prepare Configuration File**: Create a configuration file (e.g. *config-virtio-block.yaml*) specifying
the settings for the virtio block device. One example of a configuration file could be:

```
devices:
    # --- VirtIO Common ---
  - id: 0
    type: "block"
    shmem_addr: 0x50000000
    shmem_size: 0x01000000
    shmem_path: "/dev/baoipc0"
    mmio_addr: 0xa003e00
    irq: 47
    data_plane: virtio
    # --- Virtio Block Specific ---
    file_path: "/etc/block.img"
    read_only: false
    root_device: true
    advertise_flush: false
    # -----------------------------
```

2. Launch the **device model** with VirtIO Block device: 

```
nohup bao-virtio-dm --config /PATH/TO/YOUR/config-virtio-block.yaml > /etc/bao-virtio-dm.log 2>&1 &
```