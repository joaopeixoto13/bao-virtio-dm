use crate::device::VirtioDeviceT;
use crate::device::{VirtioDevType, VirtioDeviceCommon};
use crate::mmio::VIRTIO_MMIO_INT_VRING;
use crate::vhost::{VhostKernelCommon, VHOST_FEATURES};
use api::device_model::BaoDeviceModel;
use api::error::{Error, Result};
use api::types::DeviceConfig;
use event_manager::{EventManager, MutEventSubscriber};
use std::borrow::{Borrow, BorrowMut};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use vhost::vhost_kern::vsock::Vsock;
use vhost::vhost_kern::VhostKernBackend;
use vhost::vsock::VhostVsock;
use vhost::{VhostBackend, VringConfigData};
use vhost_user_frontend::GuestMemoryMmap;
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::{Queue, QueueT};
use vm_device::bus::MmioAddress;
use vm_device::device_manager::{IoManager, MmioManager};
use vm_device::MutDeviceMmio;
use vm_memory::GuestAddressSpace;

// Provides a sequenced, reliable, two-way connection-based data transmission path for datagrams of fixed
// maximum length; a consumer is required to read an entire packet with each input system call.
const VIRTIO_VSOCK_F_SEQPACKET: u64 = 1 << 1;

/// Vhost vsock device.
///
/// # Attributes
///
/// * `virtio` - Virtio common device.
/// * `vhost` - Vhost kernel common device.
/// * `vsock` - Vsock device.
/// * `guest_cid` - Guest CID.
pub struct VhostVsockDevice {
    pub virtio: VirtioDeviceCommon,
    pub vhost: VhostKernelCommon,
    pub vsock: Vsock<Arc<GuestMemoryMmap>>,
    pub guest_cid: u32,
}

impl VirtioDeviceT for VhostVsockDevice {
    fn new(
        config: &DeviceConfig,
        device_manager: Arc<Mutex<IoManager>>,
        _event_manager: Option<Arc<Mutex<EventManager<Arc<Mutex<dyn MutEventSubscriber + Send>>>>>>,
        device_model: Arc<Mutex<BaoDeviceModel>>,
    ) -> Result<Arc<Mutex<Self>>> {
        // Extract the generic features and queues.
        let (common_features, queues) = Self::initialize(&config).unwrap();

        // Extract the device features.
        let device_features = Self::device_features(&config).unwrap();

        // Update the configuration space.
        let config_space = Self::config_space(&config).unwrap();

        // Create a VirtioConfig object.
        let virtio_cfg = VirtioConfig::new(common_features | device_features, queues, config_space);

        // Create the generic device.
        let mut common_device = VirtioDeviceCommon::new(config, device_model, virtio_cfg).unwrap();

        // Extract the VirtioDeviceCommon MMIO range.
        let range = common_device.mmio.range;

        // Create the Vsock kernel device.
        let vsock_kernel = Vsock::new(Arc::new(common_device.mem())).unwrap();

        // Create the vsock device.
        let vsock = Arc::new(Mutex::new(VhostVsockDevice {
            virtio: common_device,
            vhost: VhostKernelCommon::new(device_features).unwrap(),
            vsock: vsock_kernel,
            guest_cid: config.guest_cid.unwrap() as u32,
        }));

        // Register the MMIO device within the device manager with the specified range.
        device_manager
            .lock()
            .unwrap()
            .register_mmio(range, vsock.clone())
            .unwrap();

        // Return the vosck device.
        Ok(vsock)
    }

    fn device_features(_config: &DeviceConfig) -> Result<u64> {
        Ok(VHOST_FEATURES | VIRTIO_VSOCK_F_SEQPACKET)
    }

    fn config_space(config: &DeviceConfig) -> Result<Vec<u8>> {
        // Retrieve the guest CID from the device configuration space.
        Ok(config.guest_cid.unwrap().to_le_bytes().to_vec())
    }
}

impl Borrow<VirtioConfig<Queue>> for VhostVsockDevice {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.virtio.config
    }
}

impl BorrowMut<VirtioConfig<Queue>> for VhostVsockDevice {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.virtio.config
    }
}

impl VirtioDeviceType for VhostVsockDevice {
    fn device_type(&self) -> u32 {
        VirtioDevType::Vsock as u32
    }
}

/// Implement the `VirtioDeviceActions` trait to add our custom device actions.
impl VirtioDeviceActions for VhostVsockDevice {
    type E = Error;

    // This method is called after the driver acknowledges all the device features.
    // For that reasosn, it is the right place to perform the device initialization.
    fn activate(&mut self) -> Result<()> {
        // Setup the ioeventfds by calling the generic `prepare_activate` method.
        let ioevents = self.virtio.prepare_activate().unwrap();

        // Format the queues and ioevents into a Vec<(usize, Queue, EventFd)>.
        let queues = self
            .virtio
            .config
            .queues
            .iter()
            .take(2) // The vhost vsock device has only 2 queues (RX/TX), as the Event Queue is not used.
            .enumerate()
            .zip(ioevents)
            .map(|((i, queue), ioevent)| (i, queue.clone(), ioevent))
            .collect::<Vec<_>>();

        // Set the current process as the owner of the file descriptor.
        self.vsock.set_owner().unwrap();

        // Set the device features.
        self.vsock.set_features(self.vhost.features()).unwrap();

        // Update the memory table.
        self.vsock
            .set_mem_table(self.vhost.memory(self.vsock.mem()).unwrap().as_slice())
            .unwrap();

        // Set the vring.
        let mem = self.vsock.mem();
        let mem_aux: &GuestMemoryMmap = &mem.memory();

        for (queue_index, queue, ioeventfd) in queues.iter() {
            // Set the vring num.
            self.vsock
                .set_vring_num(*queue_index, queue.size())
                .unwrap();

            let config_data = VringConfigData {
                queue_max_size: queue.max_size(),
                queue_size: queue.size(),
                flags: 0u32,
                desc_table_addr: queue.desc_table(),
                used_ring_addr: queue.used_ring(),
                avail_ring_addr: queue.avail_ring(),
                log_addr: None,
            };

            // Set the vring base.
            self.vsock
                .set_vring_base(
                    *queue_index,
                    queue.avail_idx(mem_aux, Ordering::Acquire).unwrap().0,
                )
                .unwrap();

            // Set the vring address.
            self.vsock
                .set_vring_addr(*queue_index, &config_data)
                .unwrap();

            // Set the vring call.
            self.vsock
                .set_vring_call(*queue_index, &self.virtio.irqfd.try_clone().unwrap())
                .unwrap();

            // Set the vring kick.
            self.vsock.set_vring_kick(*queue_index, ioeventfd).unwrap();
        }

        // Set the guest CID.
        self.vsock.set_guest_cid(self.guest_cid as u64).unwrap();

        // Start the vsock device.
        self.vsock.start().unwrap();

        // Set the device as activated.
        self.virtio.config.device_activated = true;

        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        // Not implemented for now.
        Ok(())
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
impl VirtioMmioDevice for VhostVsockDevice {
    fn queue_notify(&mut self, _val: u32) {
        // Do nothing, since the vhost-kernel backend device is responsible for managing the queue notifications
        // through Ioeventfds.
    }
}

/// Implement the `DeviceMmio` mutable trait to add MMIO support to our device.
/// Otherwise we could not register the device within the device manager.
impl MutDeviceMmio for VhostVsockDevice {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
