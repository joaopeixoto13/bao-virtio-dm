// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao error cases.

#![allow(dead_code)]

use std::io::Error as IoError;
use std::{io, num::ParseIntError, str};
use vmm_sys_util::errno;

/// Result code.
pub type Result<T> = std::result::Result<T, Error>;

/// Error codes.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Frontend ID {0:?}")]
    InvalidFrontendId(u16),
    #[error("Invalid MMIO {0:} Address {1:?}")]
    InvalidMmioAddr(&'static str, u64),
    #[error("MMIO Legacy not supported by Guest")]
    MmioLegacyNotSupported,
    #[error("IOMMU not supported by Guest")]
    IommuPlatformNotSupported,
    #[error("Invalid feature select {0:}")]
    InvalidFeatureSel(u32),
    #[error("Invalid MMIO direction {0:}")]
    InvalidMmioDir(u8),
    #[error("Device not supported: {0:}")]
    BaoDevNotSupported(String),
    #[error("Bao IOCTL error: {0:?} - {1:?}")]
    BaoIoctlError(io::Error, &'static str),
    #[error("Vhost user frontend error")]
    VhostFrontendError(vhost_user_frontend::Error),
    #[error("Vhost user frontend activate error")]
    VhostFrontendActivateError(vhost_user_frontend::ActivateError),
    #[error("Invalid String: {0:?}")]
    InvalidString(str::Utf8Error),
    #[error("Failed while parsing to integer: {0:?}")]
    ParseFailure(ParseIntError),
    #[error("Failed to create epoll context: {0:?}")]
    EpollCreateFd(io::Error),
    #[error("Failed to add event to epoll: {0:?}")]
    RegisterExitEvent(io::Error),
    #[error("Failed while waiting on epoll: {0:?}")]
    EpollWait(io::Error),
    #[error("Bao Bus Invalid State")]
    BaoBusInvalidState,
    #[error("Failed to kick backend: {0:?}")]
    EventFdWriteFailed(io::Error),
    #[error("Failed to open the file descriptor {0:?}: {1:?}")]
    OpenFdFailed(&'static str, io::Error),
    #[error("Invalid IO Request Direction: {0:?}")]
    InvalidIoReqDirection(u64),
    #[error("HandleIoEventFailed")]
    HandleIoEventFailed,
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Mmap guest memory failed")]
    MmapGuestMemoryFailed,
    #[error("Failed to create event manager: {0:?}")]
    EventManager(event_manager::Error),
    #[error("Failed to register the Ioeventfd: {0:?}")]
    RegisterIoevent(errno::Error),
    #[error("Failed to register the Irqfd: {0:?}")]
    RegisterIrqfd(errno::Error),
    #[error("Failed to register the Mmio")]
    MmioConfig,
    #[error("Invalid MMIO {0:?} Operation")]
    InvalidMmioOperation(&'static str),
    #[error("Device not found: {0:?} - {1:?}")]
    WrongDeviceConfiguration(String, String),
    #[error("Device bad feaures: {0:?}")]
    DeviceBadFeatures(u64),
    #[error("Device already activated")]
    DeviceAlreadyActivated,
    #[error("Failed to create the VhostUserMemoryRegion")]
    VhostUserMemoryRegion,
    #[error("Failed to create the net tap device: {0:?}")]
    NetTapCreateFailed(IoError),
    #[error("Failed to open the net tap device")]
    NetTapOpenFailed,
    #[error("Failed to set the net interface name: {0:?}")]
    NetInvalidIfname(String),
    #[error("Failed to open /dev/net/tun: {0:?}")]
    NetOpenTun(IoError),
    #[error("Ioctl error: {0:?}")]
    IoctlError(IoError),
}
