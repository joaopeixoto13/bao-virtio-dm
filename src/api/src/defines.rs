// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao defines.

#![allow(dead_code)]

/// Bao I/O Write Operation
pub const BAO_IO_WRITE: u64 = 0x0;
/// Bao I/O Read Operation
pub const BAO_IO_READ: u64 = 0x1;
/// Bao I/O Ask Operation
pub const BAO_IO_ASK: u64 = 0x2;
/// Bao I/O Notify Operation
pub const BAO_IO_NOTIFY: u64 = 0x3;

/// Bao Maximum Name Length
pub const BAO_NAME_LEN: usize = 16;

/// Bao Maximum I/O Requests
pub const BAO_IO_REQUEST_MAX: usize = 16;

/// Bao IOCTL Type
pub const BAO_IOCTL_TYPE: u32 = 0xA6;

/// Bao I/O Event File Descriptor Data Match Flag
pub const BAO_IOEVENTFD_FLAG_DATAMATCH: u32 = 1 << 1;
/// Bao I/O Event File Descriptor Deassign Flag
pub const BAO_IOEVENTFD_FLAG_DEASSIGN: u32 = 1 << 2;
/// Bao IRQ File Descriptor Assign Flag
pub const BAO_IRQFD_FLAG_ASSIGN: u32 = 0x00;
/// Bao IRQ File Descriptor Deassign Flag
pub const BAO_IRQFD_FLAG_DEASSIGN: u32 = 0x01;
