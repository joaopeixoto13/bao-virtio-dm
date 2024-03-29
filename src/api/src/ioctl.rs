// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao IOCTLs.

#![allow(dead_code)]

use super::defines::BAO_IOCTL_TYPE;
use super::types::{BaoIoEventFd, BaoIoRequest, BaoIrqFd};
use vmm_sys_util::ioctl::{_IOC_NONE, _IOC_READ, _IOC_WRITE};
use vmm_sys_util::ioctl_ioc_nr;

ioctl_ioc_nr!(
    BAO_IOCTL_VM_VIRTIO_BACKEND_CREATE,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    1 as u32,
    std::mem::size_of::<u32>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_VM_VIRTIO_BACKEND_DESTROY,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    2 as u32,
    std::mem::size_of::<u32>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_CREATE_CLIENT,
    _IOC_NONE,
    BAO_IOCTL_TYPE,
    3 as u32,
    0
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_DESTROY_CLIENT,
    _IOC_NONE,
    BAO_IOCTL_TYPE,
    4 as u32,
    0
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_ATTACH_CLIENT,
    _IOC_NONE,
    BAO_IOCTL_TYPE,
    5 as u32,
    0
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_REQUEST,
    _IOC_WRITE | _IOC_READ,
    BAO_IOCTL_TYPE,
    6 as u32,
    std::mem::size_of::<BaoIoRequest>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_REQUEST_NOTIFY_COMPLETED,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    7 as u32,
    std::mem::size_of::<BaoIoRequest>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_NOTIFY_GUEST,
    _IOC_NONE,
    BAO_IOCTL_TYPE,
    8 as u32,
    0
);
ioctl_ioc_nr!(
    BAO_IOCTL_IOEVENTFD,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    9 as u32,
    std::mem::size_of::<BaoIoEventFd>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IRQFD,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    10 as u32,
    std::mem::size_of::<BaoIrqFd>() as u32
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the BAO IOCTLs constants.
    #[test]
    fn test_ioctls() {
        assert_eq!(0x4004_A601, BAO_IOCTL_VM_VIRTIO_BACKEND_CREATE());
        assert_eq!(0x4004_A602, BAO_IOCTL_VM_VIRTIO_BACKEND_DESTROY());
        assert_eq!(0x0000_A603, BAO_IOCTL_IO_CREATE_CLIENT());
        assert_eq!(0x0000_A604, BAO_IOCTL_IO_DESTROY_CLIENT());
        assert_eq!(0x0000_A605, BAO_IOCTL_IO_ATTACH_CLIENT());
        assert_eq!(0xC048_A606, BAO_IOCTL_IO_REQUEST());
        assert_eq!(0x4048_A607, BAO_IOCTL_IO_REQUEST_NOTIFY_COMPLETED());
        assert_eq!(0x0000_A608, BAO_IOCTL_IO_NOTIFY_GUEST());
        assert_eq!(0x4020_A609, BAO_IOCTL_IOEVENTFD());
        assert_eq!(0x4008_A60A, BAO_IOCTL_IRQFD());
    }
}
