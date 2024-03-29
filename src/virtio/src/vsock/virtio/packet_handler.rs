use crate::device::SignalUsedQueue;
use api::error::{Error, Result};
use virtio_queue::{DescriptorChain, Queue, QueueOwnedT, QueueT};
use virtio_vsock::packet::VsockPacket;
use vm_memory::bitmap::AtomicBitmap;

type GuestMemoryMmap = vm_memory::GuestMemoryMmap<AtomicBitmap>;

const MAX_PKT_BUF_SIZE: u32 = 64 * 1024;

const RX_VIRTQ: usize = 0;
const TX_VIRTQ: usize = 1;

const OP_RW: u16 = 5;

pub struct VsockPacketHandler<S: SignalUsedQueue> {
    pub driver_notify: S,
    pub mem: GuestMemoryMmap,
    pub queues: Vec<Queue>,
}

impl<S> VsockPacketHandler<S>
where
    S: SignalUsedQueue,
{
    /// Process a chain.
    fn process_chain(
        &mut self,
        mut chain: DescriptorChain<&GuestMemoryMmap>,
        queue_index: usize,
    ) -> Result<()> {
        let vsock_packet;
        match queue_index {
            RX_VIRTQ => {
                vsock_packet =
                    VsockPacket::from_rx_virtq_chain(&self.mem, &mut chain, MAX_PKT_BUF_SIZE)
                        .unwrap();
                /*
                // Write data to the packet, using the setters.
                vsock_packet.set_src_cid(SRC_CID)
                    .set_dst_cid(DST_CID)
                    .set_src_port(SRC_PORT)
                    .set_dst_port(DST_PORT)
                    .set_type(TYPE_STREAM)
                    .set_buf_alloc(BUF_ALLOC)
                    .set_fwd_cnt(FWD_CNT);
                // In this example, we are sending a RW packet.
                vsock_packet.data_slice()
                    .unwrap()
                    .write_slice(&[1u8; LEN as usize], 0);
                vsock_packet.set_op(OP_RW).set_len(LEN);
                vsock_packet.header_slice().len() as u32 + LEN
                */
            }
            TX_VIRTQ => {
                vsock_packet =
                    VsockPacket::from_rx_virtq_chain(&self.mem, &mut chain, MAX_PKT_BUF_SIZE)
                        .unwrap();
                if vsock_packet.op() == OP_RW {
                    // Send the packet payload to the backend.
                }
            }
            _ => {
                println!("Invalid queue index: {}", queue_index);
                return Err(Error::DeviceNotFound);
            }
        }

        // Add the used descriptor to the queue.
        self.queues[queue_index]
            .add_used(chain.memory(), chain.head_index(), vsock_packet.len())
            .unwrap();

        // Signal the driver, if needed.
        if self.queues[queue_index]
            .needs_notification(chain.memory())
            .unwrap()
        {
            self.driver_notify.signal_used_queue(0);
        }

        Ok(())
    }

    /// Process the queue.
    ///
    /// # Arguments
    ///
    /// * `queue_index` - The index of the queue to process.
    ///
    /// # Returns
    ///
    /// * `()` - Ok if the queue was processed successfully.
    pub fn process_queue(&mut self, queue_index: usize) -> Result<()> {
        // To see why this is done in a loop, please look at the `Queue::enable_notification`
        // comments in `virtio_queue`.
        loop {
            // Disable the notifications.
            self.queues[queue_index]
                .disable_notification(&self.mem)
                .unwrap();

            // Process the queue.
            while let Some(chain) = self.queues[queue_index]
                .iter(&self.mem.clone())
                .unwrap()
                .next()
            {
                self.process_chain(chain, queue_index)?;
            }

            // Enable the notifications.
            if !self.queues[queue_index]
                .enable_notification(&self.mem)
                .unwrap()
            {
                break;
            }
        }

        Ok(())
    }
}
