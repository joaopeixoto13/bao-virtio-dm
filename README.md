# Bao Hypervisor VirtIO Device Model

## Overview

This repository offers a user space device model, implemented in the Rust programming 
language, enabling integration of various VirtIO device types within the [Bao hypervisor](https://github.com/bao-project/bao-hypervisor) 
across diverse Virtual Machines (VMs), ranging from common VirtIO devices to Vhost and 
Vhost-user backend devices.

## Getting Started

To begin utilizing VirtIO device support in Bao Hypervisor, follow these steps:

1. Clone this repository to your local environment.

```
git clone git@github.com:joaopeixoto13/bao-virtio-dm.git
```

2. Build the source code (e.g. Aarch64):

```
cargo build --target=aarch64-unknown-linux-gnu --release
```

## Supported Devices

The full list of supported (and work in progress) devices is presented below:

|                     | DEVICE            | DATAPLANE | SUPPORTED |
| ------------------- | ----------------- | -------   | --- |
| Virtio-Block        | [Block](src/virtio/src/block/README.md)            | VirtIO   | [x](src/virtio/src/block/virtio/README.md) |
| Vhost-User-Fs       | [Virtual File System](src/virtio/src/fs/README.md)            | Vhost-user   | [x](src/virtio/src/fs/vhost_user/README.md) |
| Virtio-Net        | [Net](src/virtio/src/net/README.md)            | VirtIO   | [x](src/virtio/src/net/virtio/README.md) |
| Vhost-Net        | [Net](src/virtio/src/net/README.md)            | Vhost   | - |
| Virtio-Vsock       | [Vsock](src/virtio/src/vsock/README.md)            | VirtIO   | - |
| Vhost-Vsock       | [Vsock](src/virtio/src/vsock/README.md)            | Vhost   | [x](src/virtio/src/vsock/vhost/README.md) |
| Vhost-User-Vsock       | [Vsock](src/virtio/src/vsock/README.md)            | Vhost-user   | [x](src/virtio/src/vsock/vhost_user/README.md) |

## Contributing
Contributions to enhance the functionality and features of Bao Hypervisor VirtIO Device 
Support are welcome. If you have suggestions, bug fixes, or new features to propose, 
feel free to open an issue or submit a pull request.