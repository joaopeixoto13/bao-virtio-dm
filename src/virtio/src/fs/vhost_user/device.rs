use crate::device::{SingleFdSignalQueue, VirtioDeviceT};
use crate::device::{VirtioDevType, VirtioDeviceCommon};
use crate::mmio::VIRTIO_MMIO_INT_VRING;
use api::device_model::BaoDeviceModel;
use api::error::{Error, Result};
use api::types::DeviceConfig;
use event_manager::{EventManager, MutEventSubscriber};
use seccompiler::SeccompAction;
use std::borrow::{Borrow, BorrowMut};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use vhost::vhost_user::message::VhostUserProtocolFeatures;
use vhost_user_frontend::{
    Generic as VhostUserCommon, VhostUserConfig, VirtioDevice,
    VirtioDeviceType as VhostUserDeviceType,
};
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::{Queue, QueueT};
use vm_device::bus::MmioAddress;
use vm_device::device_manager::{IoManager, MmioManager};
use vm_device::MutDeviceMmio;
use vm_memory::GuestMemoryAtomic;
use vmm_sys_util::eventfd::{EventFd, EFD_NONBLOCK};

/// Vhost-user file system device.
///
/// # Attributes
///
/// * `vhost_user` - Vhost-user generic device.
/// * `virtio` - Virtio virtio device.
/// * `socket_path` - Path to the vhost-user socket.
pub struct VhostUserFs {
    pub virtio: VirtioDeviceCommon,
    pub vhost_user: Mutex<VhostUserCommon>,
    pub socket_path: String,
}

impl VirtioDeviceT for VhostUserFs {
    fn new(
        config: &DeviceConfig,
        device_manager: Arc<Mutex<IoManager>>,
        event_manager: Arc<Mutex<EventManager<Arc<Mutex<dyn MutEventSubscriber + Send>>>>>,
        device_model: Arc<Mutex<BaoDeviceModel>>,
    ) -> Result<Arc<Mutex<Self>>> {
        // Extract the generic features and queues.
        let (common_features, queues) = Self::initialize(&config).unwrap();

        // Update the configuration space.
        let config_space = Self::config_space(&config).unwrap();

        // Create the vhost-user configuration.
        let vu_cfg = VhostUserConfig {
            socket: format!(
                "{}{}.sock",
                config.socket_path.as_ref().unwrap(),
                VirtioDevType::from(VirtioDevType::Fs).to_string()
            ),
            num_queues: queues.len(),
            queue_size: queues[0].size(),
        };

        println!(
            "Connecting to {} device backend over {} socket..",
            VirtioDevType::from(VirtioDevType::Fs).to_string(),
            vu_cfg.socket
        );

        // Create the VhostUserCommon vhost-user device.
        let vhost_user = VhostUserCommon::new(
            vu_cfg,
            SeccompAction::Allow,
            EventFd::new(EFD_NONBLOCK).unwrap(),
            VhostUserDeviceType::Fs,
        )
        .map_err(Error::VhostFrontendError)?;

        println!(
            "Connected to {} device backend.",
            VirtioDevType::from(VirtioDevType::Fs).to_string()
        );

        // Update the device features since we have the vhost-user backend now.
        let device_features = Self::device_features(&config).unwrap()
            | common_features
            | vhost_user.device_features();

        // Create a VirtioConfig object.
        let virtio_cfg = VirtioConfig::new(device_features, queues, config_space);

        // Create the generic device.
        let common_device =
            VirtioDeviceCommon::new(config, event_manager, device_model, virtio_cfg).unwrap();

        // Extract the VirtioDeviceCommon MMIO range.
        let range = common_device.mmio.range;

        // Create the fs device.
        let fs = Arc::new(Mutex::new(VhostUserFs {
            vhost_user: Mutex::new(vhost_user),
            virtio: common_device,
            socket_path: config.socket_path.clone().unwrap(),
        }));

        // Register the MMIO device within the device manager with the specified range.
        device_manager
            .lock()
            .unwrap()
            .register_mmio(range, fs.clone())
            .unwrap();

        // Return the fs device.
        Ok(fs)
    }

    fn device_features(_config: &DeviceConfig) -> Result<u64> {
        // Here we can leave empty since it is the vhost-user backend responsibility to negotiate the features
        // that it supports.
        Ok(0)
    }

    fn config_space(_config: &DeviceConfig) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

impl Borrow<VirtioConfig<Queue>> for VhostUserFs {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.virtio.config
    }
}

impl BorrowMut<VirtioConfig<Queue>> for VhostUserFs {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.virtio.config
    }
}

impl VirtioDeviceType for VhostUserFs {
    fn device_type(&self) -> u32 {
        VirtioDevType::Fs as u32
    }
}

/// Implement the `VirtioDeviceActions` trait to add our custom device actions.
impl VirtioDeviceActions for VhostUserFs {
    type E = Error;

    // This method is called after the driver acknowledges all the device features.
    // For that reasosn, it is the right place to perform the device initialization.
    fn activate(&mut self) -> Result<()> {
        // Setup the ioeventfds by calling the generic `prepare_activate` method.
        let ioevents = self.virtio.prepare_activate().unwrap();

        // Create the driver notify object.
        let driver_notify = SingleFdSignalQueue {
            irqfd: self.virtio.irqfd.try_clone().unwrap(),
            interrupt_status: self.virtio.config.interrupt_status.clone(),
        };

        // Format the queues and ioevents into a Vec<(usize, Queue, EventFd)>.
        let queues = self
            .virtio
            .config
            .queues
            .iter()
            .enumerate()
            .zip(ioevents)
            .map(|((i, queue), ioevent)| (i, queue.clone(), ioevent))
            .collect::<Vec<_>>();

        // Activate the vhost-user device.
        self.vhost_user
            .lock()
            .unwrap()
            .activate(
                GuestMemoryAtomic::new(self.virtio.mem()),
                Arc::new(driver_notify),
                queues,
            )
            .unwrap();

        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        // Not implemented for now.
        Ok(())
    }

    // This method is called when the driver wants to read information from the device configuration space.
    // Since the device configuration space is managed by the device and the device can be implemented in
    // different handlers outside of the VMM (vhost or vhost-user) we need to invoke dedicated logic.
    fn read_config(&self, offset: usize, data: &mut [u8]) {
        self.vhost_user
            .lock()
            .unwrap()
            .read_config(offset as u64, data);
    }

    // This method is called when the driver wants to write information to the device configuration space.
    // Since the device configuration space is managed by the device and the device can be implemented in
    // different handlers outside of the VMM (vhost or vhost-user) we need to invoke dedicated logic.
    fn write_config(&mut self, offset: usize, data: &[u8]) {
        self.vhost_user
            .lock()
            .unwrap()
            .write_config(offset as u64, data);
    }

    // This method is called when the driver finishes the negotiation of the device features
    // with the frontend device (selecting page 0). This method is crucial when the device handlers are
    // implemented outside of the VMM (vhost or vhost-user) as the frontend device needs to negotiate the
    // features with the backend device. Otherwise, the device is not prepared to support, for example,
    // multiple queues and configuration space reads and writes.
    fn negotiate_driver_features(&mut self) {
        self.vhost_user
            .lock()
            .unwrap()
            .negotiate_features(
                self.virtio.config.driver_features,
                VhostUserProtocolFeatures::empty(),
            )
            .unwrap();
    }

    // This method is called when the driver needs to read the interrupt status from the device.
    // Since it's the frontend device responsibility to manage the interrupt status, we need to invoke
    // dedicated logic to update the interrupt status accordingly (Used Buffer Notification or Configuration Change Notification).
    // Note: If the device is implemented in the VMM, the interrupt status can be managed and updated directly by the device.
    fn interrupt_status(&self) -> &Arc<AtomicU8> {
        // We assume that all the interrupts are Used Buffer Notifications.
        self.virtio
            .config
            .interrupt_status
            .fetch_or(VIRTIO_MMIO_INT_VRING, Ordering::SeqCst);
        &self.virtio.config.interrupt_status
    }
}

/// Implement the `VirtioMmioDevice` trait to add VirtIO MMIO support to our device.
impl VirtioMmioDevice for VhostUserFs {
    fn queue_notify(&mut self, _val: u32) {
        // Do nothing, since the vhost-user backend device is responsible for managing the queue notifications.
        // through Ioeventfds.
    }
}

/// Implement the `DeviceMmio` mutable trait to add MMIO support to our device.
/// Otherwise we could not register the device within the device manager.
impl MutDeviceMmio for VhostUserFs {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
