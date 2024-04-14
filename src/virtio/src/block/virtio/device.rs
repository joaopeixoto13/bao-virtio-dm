use crate::device::{VirtioDevType, VirtioDeviceCommon};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

use super::inorder_handler::InOrderQueueHandler;
use super::queue_handler::QueueHandler;
use crate::device::{SingleFdSignalQueue, VirtioDeviceT};
use api::device_model::BaoDeviceModel;
use api::error::{Error, Result};
use api::types::DeviceConfig;
use event_manager::{EventManager, MutEventSubscriber};
use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex};
use virtio_bindings::virtio_blk::{VIRTIO_BLK_F_FLUSH, VIRTIO_BLK_F_RO};
use virtio_blk::stdio_executor::StdIoBackend;
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::Queue;
use vm_device::bus::MmioAddress;
use vm_device::device_manager::{IoManager, MmioManager};
use vm_device::MutDeviceMmio;

// The sector size is 512 bytes (1 << 9).
const SECTOR_SHIFT: u8 = 9;

/// Virtio block device.
///
/// # Attributes
///
/// * `common` - Virtio common device.
/// * `file_path` - Path to the block device file or disk partition.
/// * `read_only` - Whether the block device is read-only.
/// * `root_device` - Whether the block device is the root device.
/// * `advertise_flush` - Whether the block device advertises the flush feature.
pub struct VirtioBlock {
    pub common: VirtioDeviceCommon,
    pub file_path: PathBuf,
    pub read_only: bool,
    pub root_device: bool,
    pub advertise_flush: bool,
}

impl VirtioDeviceT for VirtioBlock {
    fn new(
        config: &DeviceConfig,
        device_manager: Arc<Mutex<IoManager>>,
        event_manager: Arc<Mutex<EventManager<Arc<Mutex<dyn MutEventSubscriber + Send>>>>>,
        device_model: Arc<Mutex<BaoDeviceModel>>,
    ) -> Result<Arc<Mutex<Self>>> {
        // Extract the generic features and queues.
        let (common_features, queues) = Self::initialize(&config).unwrap();

        // Update the device features.
        let device_features = common_features | Self::device_features(&config).unwrap();

        // Update the configuration space.
        let config_space = Self::config_space(&config).unwrap();

        // Create a VirtioConfig object.
        let virtio_cfg = VirtioConfig::new(device_features, queues, config_space);

        // Create the generic device.
        let common_device =
            VirtioDeviceCommon::new(config, event_manager, device_model, virtio_cfg).unwrap();

        // Create the block device.
        let block = Arc::new(Mutex::new(VirtioBlock {
            common: common_device,
            file_path: config.file_path.clone().unwrap().into(),
            read_only: config.read_only.unwrap(),
            root_device: config.root_device.unwrap(),
            advertise_flush: config.advertise_flush.unwrap(),
        }));

        // Register the MMIO device within the device manager with the specified range.
        device_manager
            .lock()
            .unwrap()
            .register_mmio(
                block.clone().lock().unwrap().common.mmio.range,
                block.clone(),
            )
            .unwrap();

        // Return the block device.
        Ok(block)
    }

    fn device_features(config: &DeviceConfig) -> Result<u64> {
        let mut features = 0;

        // Set the read-only feature.
        if config.read_only.unwrap() {
            features |= 1 << VIRTIO_BLK_F_RO;
        }

        // Set the flush feature.
        if config.advertise_flush.unwrap() {
            features |= 1 << VIRTIO_BLK_F_FLUSH;
        }

        Ok(features)
    }

    fn config_space(config: &DeviceConfig) -> Result<Vec<u8>> {
        // TODO: right now, the file size is computed by the StdioBackend as well. Maybe we should
        // create the backend as early as possible, and get the size information from there.
        let file_size = File::open(config.file_path.clone().unwrap())
            .unwrap()
            .seek(SeekFrom::End(0))
            .unwrap();

        // If the file size is actually not a multiple of sector size, then data at the very end
        // will be ignored.
        let num_sectors = file_size >> SECTOR_SHIFT;

        // Update the configuration space.
        // This must be little-endian according to the Virtio specification.
        Ok(num_sectors.to_le_bytes().to_vec())
    }
}

impl Borrow<VirtioConfig<Queue>> for VirtioBlock {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.common.config
    }
}

impl BorrowMut<VirtioConfig<Queue>> for VirtioBlock {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.common.config
    }
}

impl VirtioDeviceType for VirtioBlock {
    fn device_type(&self) -> u32 {
        VirtioDevType::Block as u32
    }
}

/// Implement the `VirtioDeviceActions` trait to add our custom device actions.
impl VirtioDeviceActions for VirtioBlock {
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        // Open the block device file.
        let file = OpenOptions::new()
            .read(true)
            .write(!self.read_only)
            .open(&self.file_path)
            .unwrap();

        // Create the backend.
        // TODO: Create the backend earlier (as part of `VirtioBlock::new`)?
        let disk = StdIoBackend::new(file, self.common.config.driver_features).unwrap();

        // Create the driver notify object.
        let driver_notify = SingleFdSignalQueue {
            irqfd: self.common.irqfd.try_clone().unwrap(),
            interrupt_status: self.common.config.interrupt_status.clone(),
        };

        // Prepare the activation by calling the generic `prepare_activate` method.
        let mut ioevents = self.common.prepare_activate().unwrap();

        // Create the inner handler.
        let inner = InOrderQueueHandler {
            driver_notify,
            mem: self.common.mem(),
            queue: self.common.config.queues.remove(0),
            disk,
        };

        // Create the queue handler.
        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            ioeventfd: ioevents.remove(0),
        }));

        // Finalize the activation by calling the generic `finalize_activate` method.
        let ret = self.common.finalize_activate(handler);

        Ok(ret.unwrap())
    }

    fn reset(&mut self) -> Result<()> {
        // Not implemented for now.
        Ok(())
    }
}

/// Implement the `VirtioMmioDevice` trait to add VirtIO MMIO support to our device.
impl VirtioMmioDevice for VirtioBlock {
    fn queue_notify(&mut self, _val: u32) {
        // Do nothing for now.
    }
}

/// Implement the `DeviceMmio` mutable trait to add MMIO support to our device.
/// Otherwise we could not register the device within the device manager.
impl MutDeviceMmio for VirtioBlock {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
