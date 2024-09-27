use super::console_handler::ConsoleQueueHandler;
use super::queue_handler::QueueHandler;
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
use virtio_bindings::virtio_config::VIRTIO_F_IN_ORDER;
use virtio_console::console::Console;
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::Queue;
use vm_device::bus::MmioAddress;
use vm_device::device_manager::{IoManager, MmioManager};
use vm_device::MutDeviceMmio;

/// Virtio console device.
///
/// # Attributes
///
/// * `common` - Virtio common device.
/// * `endpoint` - The remote subscriber endpoint.
pub struct VirtioConsole {
    pub common: VirtioDeviceCommon,
    pub endpoint: RemoteEndpoint<Subscriber>,
}

impl VirtioDeviceT for VirtioConsole {
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

        // Create the console device.
        let console = Arc::new(Mutex::new(VirtioConsole {
            common: common_device,
            endpoint: remote_endpoint,
        }));

        // Register the MMIO device within the device manager with the specified range.
        device_manager
            .lock()
            .unwrap()
            .register_mmio(
                console.clone().lock().unwrap().common.mmio.range,
                console.clone(),
            )
            .unwrap();

        // Return the console device.
        Ok(console)
    }

    fn device_features(_config: &DeviceConfig) -> Result<u64> {
        Ok(1 << VIRTIO_F_IN_ORDER)
    }

    fn config_space(_config: &DeviceConfig) -> Result<Vec<u8>> {
        // https://docs.oasis-open.org/virtio/virtio/v1.3/csd01/virtio-v1.3-csd01.html#x1-3210003
        let cols: u16 = 80;
        let rows: u16 = 25;
        let max_nr_ports: u32 = 1;
        let mut config = Vec::new();
        config.extend_from_slice(&cols.to_le_bytes());
        config.extend_from_slice(&rows.to_le_bytes());
        config.extend_from_slice(&max_nr_ports.to_le_bytes());
        Ok(config)
    }
}

impl Borrow<VirtioConfig<Queue>> for VirtioConsole {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.common.config
    }
}

impl BorrowMut<VirtioConfig<Queue>> for VirtioConsole {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.common.config
    }
}

impl VirtioDeviceType for VirtioConsole {
    fn device_type(&self) -> u32 {
        VirtioDevType::Console as u32
    }
}

/// Implement the `VirtioDeviceActions` trait to add our custom device actions.
impl VirtioDeviceActions for VirtioConsole {
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        // Create the backend.
        let console = Console::default();

        // Create the driver notify object.
        let driver_notify = SingleFdSignalQueue {
            irqfd: self.common.irqfd.try_clone().unwrap(),
            interrupt_status: self.common.config.interrupt_status.clone(),
        };

        // Prepare the activation by calling the generic `prepare_activate` method.
        let mut ioevents = self.common.prepare_activate().unwrap();

        // Create the inner handler.
        let inner = ConsoleQueueHandler {
            driver_notify,
            mem: self.common.mem(),
            input_queue: self.common.config.queues.remove(0),
            output_queue: self.common.config.queues.remove(0),
            console,
        };

        // Create the queue handler.
        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            input_ioeventfd: ioevents.remove(0),
            output_ioeventfd: ioevents.remove(0),
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
impl VirtioMmioDevice for VirtioConsole {
    fn queue_notify(&mut self, _val: u32) {
        // Do nothing for now.
    }
}

/// Implement the `DeviceMmio` mutable trait to add MMIO support to our device.
/// Otherwise we could not register the device within the device manager.
impl MutDeviceMmio for VirtioConsole {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
