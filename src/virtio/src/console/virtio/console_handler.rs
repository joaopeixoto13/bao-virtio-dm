use super::queue_handler::{INPUT_QUEUE_INDEX, OUTPUT_QUEUE_INDEX};
use crate::device::SignalUsedQueue;
use std::io::Stdout;
use std::result;
use virtio_console::console::{Console, Error as ConsoleError};
use virtio_queue::{Queue, QueueOwnedT, QueueT};
use vm_memory::bitmap::AtomicBitmap;

type GuestMemoryMmap = vm_memory::GuestMemoryMmap<AtomicBitmap>;

pub struct ConsoleQueueHandler<S: SignalUsedQueue> {
    pub driver_notify: S,
    pub mem: GuestMemoryMmap,
    pub input_queue: Queue,
    pub output_queue: Queue,
    pub console: Console<Stdout>,
}

impl<S> ConsoleQueueHandler<S>
where
    S: SignalUsedQueue,
{
    /*
     * Each port of virtio console device has one receive
     * queue. One or more empty buffers are placed by the
     * driver in the receive queue for incoming data. Here,
     * we place the input data to these empty buffers.
     */
    pub fn process_input_queue(&mut self) -> result::Result<(), Error> {
        // To see why this is done in a loop, please look at the `Queue::enable_notification`
        // comments in `virtio_queue`.
        loop {
            // Disable the notifications.
            self.input_queue.disable_notification(&self.mem)?;

            // Process the queue.
            while let Some(mut chain) = self.input_queue.iter(&self.mem.clone())?.next() {
                let sent_bytes = self.console.process_receiveq_chain(&mut chain)?;
                //println!("Sent bytes: {}", sent_bytes);

                if sent_bytes > 0 {
                    println!("process_input_queue bytes: {}", sent_bytes);
                    self.input_queue
                        .add_used(chain.memory(), chain.head_index(), sent_bytes)?;

                    if self.input_queue.needs_notification(&self.mem)? {
                        self.driver_notify.signal_used_queue(INPUT_QUEUE_INDEX);
                    }
                }
            }

            // Enable the notifications.
            if !self.input_queue.enable_notification(&self.mem)? {
                break;
            }
        }
        Ok(())
    }

    /*
     * Each port of virtio console device has one transmit
     * queue. For outgoing data, characters are placed in
     * the transmit queue by the driver. Therefore, here
     * we read data from the transmit queue and flush them
     * to the referenced address.
     */
    pub fn process_output_queue(&mut self) -> result::Result<(), Error> {
        // To see why this is done in a loop, please look at the `Queue::enable_notification`
        // comments in `virtio_queue`.
        loop {
            // Disable the notifications.
            self.output_queue.disable_notification(&self.mem)?;

            // Process the queue.
            while let Some(mut chain) = self.output_queue.iter(&self.mem.clone())?.next() {
                self.console.process_transmitq_chain(&mut chain)?;

                self.output_queue
                    .add_used(chain.memory(), chain.head_index(), 0)?;

                if self.output_queue.needs_notification(&self.mem)? {
                    self.driver_notify.signal_used_queue(OUTPUT_QUEUE_INDEX);
                }
            }

            // Enable the notifications.
            if !self.output_queue.enable_notification(&self.mem)? {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    GuestMemory(vm_memory::GuestMemoryError),
    Queue(virtio_queue::Error),
    ConsoleError(ConsoleError),
}

impl From<vm_memory::GuestMemoryError> for Error {
    fn from(e: vm_memory::GuestMemoryError) -> Self {
        Error::GuestMemory(e)
    }
}

impl From<virtio_queue::Error> for Error {
    fn from(e: virtio_queue::Error) -> Self {
        Error::Queue(e)
    }
}

impl From<ConsoleError> for Error {
    fn from(e: ConsoleError) -> Self {
        Error::ConsoleError(e)
    }
}
