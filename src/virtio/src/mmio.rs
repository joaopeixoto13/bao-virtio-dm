use virtio_bindings::virtio_mmio::VIRTIO_MMIO_QUEUE_NOTIFY;
use vm_device::bus::{self, MmioAddress, MmioRange};

#[derive(Debug)]
pub enum Error {
    Bus(bus::Error),
    Overflow,
}

/// A specialized `Result` type for MMIO operations.
type Result<T> = std::result::Result<T, Error>;

/// The offset of the queue notify register.
pub const VIRTIO_MMIO_QUEUE_NOTIFY_OFFSET: u64 = VIRTIO_MMIO_QUEUE_NOTIFY as u64;

/// This bit is set on the device interrupt status when notifying the driver about used
/// queue events (Used Buffer Notification).
pub const VIRTIO_MMIO_INT_VRING: u8 = 0x01;

/// This bit is set on the device interrupt status when the device configuration has changed
/// (Configuration Change Notification).
pub const VIRTIO_MMIO_INT_CONFIG: u8 = 0x02;

/// Represents the configuration of a MMIO device.
///
/// # Attributes
///
/// * `range` - The MMIO range assigned to the device.
/// * `gsi` - The interrupt assigned to the device.
#[derive(Copy, Clone)]
pub struct MmioConfig {
    pub range: MmioRange,
    pub gsi: u32,
}

impl MmioConfig {
    /// Creates a new `MmioConfig` object.
    ///
    /// # Arguments
    ///
    /// * `base` - The base address of the MMIO range.
    /// * `size` - The size of the MMIO range.
    /// * `gsi` - The interrupt assigned to the device.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `MmioConfig` object.
    pub fn new(base: u64, size: u64, gsi: u32) -> Result<Self> {
        MmioRange::new(MmioAddress(base), size)
            .map(|range| MmioConfig { range, gsi })
            .map_err(Error::Bus)
    }

    /// Returns the next `MmioConfig` object.
    ///
    /// # Returns
    ///
    /// A `Result` containing the next `MmioConfig` object.
    pub fn next(&self) -> Result<Self> {
        let range = self.range;
        let next_start = range
            .base()
            .0
            .checked_add(range.size())
            .ok_or(Error::Overflow)?;
        Self::new(next_start, range.size(), self.gsi + 1)
    }
}
