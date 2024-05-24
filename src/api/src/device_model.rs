// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao device model.

use crate::defines::{BAO_IO_ASK, BAO_IRQFD_FLAG_ASSIGN};
use crate::error::{Error, Result};
use crate::ioctl::*;
use crate::types::{BaoIoEventFd, BaoIoRequest, BaoIrqFd};
use libc::ioctl;
use std::os::fd::AsRawFd;
use vmm_sys_util::errno;
use vmm_sys_util::eventfd::EventFd;

/// Bao Hypervisor Device Model.
///
/// # Attributes
///
/// * `fd` - The file descriptor for the VMM.
/// * `devmodel_fd` - The file descriptor for the device model.
/// * `id` - The ID of the device model.
#[derive(Clone)]
pub struct BaoDeviceModel {
    pub fd: i32,
    pub devmodel_fd: i32,
    pub id: u16,
}

impl BaoDeviceModel {
    /// Create a new device model.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor for the VMM.
    /// * `id` - The ID of the device model.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn new(fd: i32, id: u16) -> Result<Self> {
        let devmodel_fd;

        unsafe {
            devmodel_fd = ioctl(fd, BAO_IOCTL_VM_VIRTIO_BACKEND_CREATE(), &(id as i32));

            if devmodel_fd < 0 {
                return Err(Error::OpenFdFailed(
                    "devmodel_fd",
                    std::io::Error::last_os_error(),
                ));
            }
        }

        // Create the device model object.
        let device_model = BaoDeviceModel {
            fd,
            devmodel_fd,
            id,
        };

        Ok(device_model)
    }

    /// Destroy the device model.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn destroy(&self) -> Result<()> {
        unsafe {
            let ret = ioctl(
                self.devmodel_fd,
                BAO_IOCTL_VM_VIRTIO_BACKEND_DESTROY(),
                &(self.id as i32),
            );

            if ret < 0 {
                return Err(Error::OpenFdFailed(
                    "guest_fd",
                    std::io::Error::last_os_error(),
                ));
            }
        }
        Ok(())
    }

    /// Attach the I/O client to the VM.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn attach_io_client(&self) -> Result<BaoIoRequest> {
        // Create a new I/O request
        let mut request = BaoIoRequest {
            virtio_id: 0,
            addr: 0,
            op: BAO_IO_ASK,
            value: 0,
            access_width: 0,
            cpu_id: 0,
            vcpu_id: 0,
            ret: 0,
        };
        unsafe {
            let ret = ioctl(self.devmodel_fd, BAO_IOCTL_IO_ATTACH_CLIENT(), &mut request);

            if ret < 0 {
                return Err(Error::BaoIoctlError(
                    std::io::Error::last_os_error(),
                    std::any::type_name::<Self>(),
                ));
            }
        }
        Ok(request)
    }

    /// Notifies I/O request completion.
    ///
    /// # Arguments
    ///
    /// * `req` - The BaoIoRequest to be notified.
    ///
    /// # Return
    ///
    /// * `Result<()>` - A Result containing Ok(()) on success, or an Error on failure.
    pub fn notify_io_completed(&self, req: BaoIoRequest) -> Result<()> {
        // Notify I/O request completion
        unsafe {
            let ret = ioctl(
                self.devmodel_fd,
                BAO_IOCTL_IO_REQUEST_NOTIFY_COMPLETED(),
                &req,
            );

            if ret < 0 {
                return Err(Error::BaoIoctlError(
                    std::io::Error::last_os_error(),
                    std::any::type_name::<Self>(),
                ));
            }
        }

        // Return Ok(()) on success
        Ok(())
    }

    /// Notifies the guest about a Used Buffer Notification or
    /// a Configuration Change Notification.
    ///
    /// # Return
    ///
    /// * `Result<()>` - A Result containing Ok(()) on success, or an Error on failure.
    pub fn notify_guest(&self) -> Result<()> {
        // Notify the guest
        unsafe {
            let ret = ioctl(self.devmodel_fd, BAO_IOCTL_IO_NOTIFY_GUEST());

            if ret < 0 {
                return Err(Error::BaoIoctlError(
                    std::io::Error::last_os_error(),
                    std::any::type_name::<Self>(),
                ));
            }
        }

        // Return Ok(()) on success
        Ok(())
    }

    /// Registers an ioeventfd within the VM (guest to host interrupt)
    ///
    /// # Arguments
    ///
    /// * `kick` - The EventFd to be registered.
    /// * `flags` - The flags to be used.
    /// * `addr` - The address to be registered.
    /// * `datamatch` - The data to be matched (index of the Virtqueue).
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn register_ioeventfd(
        &self,
        kick: u32,
        flags: u32,
        addr: u64,
        datamatch: u64,
    ) -> Result<()> {
        // Create a BaoIoEventFd struct.
        let ioeventfd = BaoIoEventFd {
            fd: kick,
            flags: flags,
            addr: addr,
            len: 4,
            reserved: 0,
            data: datamatch, // Index of the Virtqueue to match with the 'value' field of the 'bao_io_request' struct
        };

        // Call the ioctl to register the ioeventfd.
        unsafe {
            let ret = ioctl(self.devmodel_fd, BAO_IOCTL_IOEVENTFD(), &ioeventfd);

            if ret < 0 {
                return Err(Error::RegisterIoevent(errno::Error::last()));
            }
        }
        Ok(())
    }

    /// Registers an irqfd within the VM (host to guest interrupt)
    ///
    /// # Arguments
    ///
    /// * `call` - The EventFd to be registered.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn register_irqfd(&self, call: &EventFd) -> Result<()> {
        // Create a BaoIrqFd struct.
        let irqfd = BaoIrqFd {
            fd: call.as_raw_fd() as i32,
            flags: BAO_IRQFD_FLAG_ASSIGN, // Assign the Irqfd
        };

        // Call the ioctl to register the irqfd.
        unsafe {
            let ret = ioctl(self.devmodel_fd, BAO_IOCTL_IRQFD(), &irqfd);

            if ret < 0 {
                return Err(Error::RegisterIrqfd(errno::Error::last()));
            }
        }
        Ok(())
    }
}
