// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao IOCTLs.

#![allow(dead_code)]

use crate::types::BaoDMInfo;

use super::defines::BAO_IOCTL_TYPE;
use super::types::{BaoIoEventFd, BaoIoRequest, BaoIrqFd};
use vmm_sys_util::ioctl::{_IOC_READ, _IOC_WRITE};
use vmm_sys_util::ioctl_ioc_nr;

ioctl_ioc_nr!(
    BAO_IOCTL_IO_DM_GET_INFO,
    _IOC_WRITE | _IOC_READ,
    BAO_IOCTL_TYPE,
    1 as u32,
    std::mem::size_of::<BaoDMInfo>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_ATTACH_CLIENT,
    _IOC_WRITE | _IOC_READ,
    BAO_IOCTL_TYPE,
    2 as u32,
    std::mem::size_of::<BaoIoRequest>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IO_REQUEST_NOTIFY_COMPLETED,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    3 as u32,
    std::mem::size_of::<BaoIoRequest>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IOEVENTFD,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    4 as u32,
    std::mem::size_of::<BaoIoEventFd>() as u32
);
ioctl_ioc_nr!(
    BAO_IOCTL_IRQFD,
    _IOC_WRITE,
    BAO_IOCTL_TYPE,
    5 as u32,
    std::mem::size_of::<BaoIrqFd>() as u32
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the BAO IOCTLs constants.
    #[test]
    fn test_ioctls() {
        assert_eq!(0xC040_A601, BAO_IOCTL_IO_DM_GET_INFO());
        assert_eq!(0xC040_A602, BAO_IOCTL_IO_ATTACH_CLIENT());
        assert_eq!(0x4040_A603, BAO_IOCTL_IO_REQUEST_NOTIFY_COMPLETED());
        assert_eq!(0x4020_A604, BAO_IOCTL_IOEVENTFD());
        assert_eq!(0x4008_A605, BAO_IOCTL_IRQFD());
    }
}
