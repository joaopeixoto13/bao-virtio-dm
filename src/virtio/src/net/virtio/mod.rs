pub mod bindings;
pub mod device;
mod queue_handler;
mod simple_handler;
pub mod tap;

// Size of the `virtio_net_hdr` (VirtIO Net header) structure defined by the standard.
pub const VIRTIO_NET_HDR_SIZE: usize = 12;
