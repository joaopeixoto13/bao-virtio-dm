use super::packet_handler::VsockPacketHandler;
use super::queue_handler::QueueHandler;
use crate::device::clone_queue;
use crate::device::{SingleFdSignalQueue, Subscriber, VirtioDeviceT};
use crate::device::{VirtioDevType, VirtioDeviceCommon};
use api::device_model::BaoDeviceModel;
use api::error::{Error, Result};
use api::types::DeviceConfig;
use event_manager::{
    EventManager, MutEventSubscriber, RemoteEndpoint, Result as EvmgrResult, SubscriberId,
};
use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex};
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::Queue;
use vm_device::bus::MmioAddress;
use vm_device::device_manager::{IoManager, MmioManager};
use vm_device::MutDeviceMmio;

/// Virtio vsock device.
///
/// # Attributes
///
/// * `common` - Virtio common device.
/// * `endpoint` - The remote subscriber endpoint.
/// * `guest_cid` - The guest CID.
pub struct VirtioVsock {
    pub common: VirtioDeviceCommon,
    pub endpoint: RemoteEndpoint<Subscriber>,
    pub guest_cid: u64,
}

impl VirtioDeviceT for VirtioVsock {
    fn new(
        config: &DeviceConfig,
        device_manager: Arc<Mutex<IoManager>>,
        event_manager: Option<Arc<Mutex<EventManager<Arc<Mutex<dyn MutEventSubscriber + Send>>>>>>,
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
        let common_device = VirtioDeviceCommon::new(config, device_model, virtio_cfg).unwrap();

        // Create a remote endpoint object, that allows interacting with the VM EventManager from a different thread.
        let remote_endpoint = event_manager.unwrap().lock().unwrap().remote_endpoint();

        // Create the vsock device.
        let vsock = Arc::new(Mutex::new(VirtioVsock {
            common: common_device,
            endpoint: remote_endpoint,
            guest_cid: config.guest_cid.unwrap(),
        }));

        // Register the MMIO device within the device manager with the specified range.
        device_manager
            .lock()
            .unwrap()
            .register_mmio(
                vsock.clone().lock().unwrap().common.mmio.range,
                vsock.clone(),
            )
            .unwrap();

        // Return the vsock device.
        Ok(vsock)
    }

    fn device_features(_config: &DeviceConfig) -> Result<u64> {
        Ok(0)
    }

    fn config_space(_config: &DeviceConfig) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

impl Borrow<VirtioConfig<Queue>> for VirtioVsock {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.common.config
    }
}

impl BorrowMut<VirtioConfig<Queue>> for VirtioVsock {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.common.config
    }
}

impl VirtioDeviceType for VirtioVsock {
    fn device_type(&self) -> u32 {
        VirtioDevType::Vsock as u32
    }
}

/// Implement the `VirtioDeviceActions` trait to add our custom device actions.
impl VirtioDeviceActions for VirtioVsock {
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        // Create the driver notify object.
        let driver_notify = SingleFdSignalQueue {
            irqfd: self.common.irqfd.try_clone().unwrap(),
            interrupt_status: self.common.config.interrupt_status.clone(),
        };

        // Prepare the activation by calling the generic `prepare_activate` method.
        let ioevents = self.common.prepare_activate().unwrap();

        // Clone the queues.
        let queues = self
            .common
            .config
            .queues
            .iter()
            .map(|queue| (clone_queue(&queue)))
            .collect::<Vec<_>>();

        // Create the inner handler.
        let inner = VsockPacketHandler {
            driver_notify,
            mem: self.common.mem(),
            queues: queues,
        };

        // Create the queue handler.
        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            ioeventfd: ioevents,
        }));

        // Register the queue handler with the `EventManager`. We could record the `sub_id`
        // (and/or keep a handler clone) for further interaction (i.e. to remove the subscriber at
        // a later time, retrieve state, etc).
        let _sub_id = self
            .endpoint
            .call_blocking(move |mgr| -> EvmgrResult<SubscriberId> {
                Ok(mgr.add_subscriber(handler))
            })
            .unwrap();

        // Set the device as activated.
        self.common.config.device_activated = true;

        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        // Not implemented for now.
        Ok(())
    }
}

/// Implement the `VirtioMmioDevice` trait to add VirtIO MMIO support to our device.
impl VirtioMmioDevice for VirtioVsock {
    fn queue_notify(&mut self, _val: u32) {
        // Do nothing for now.
    }
}

/// Implement the `DeviceMmio` mutable trait to add MMIO support to our device.
/// Otherwise we could not register the device within the device manager.
impl MutDeviceMmio for VirtioVsock {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
