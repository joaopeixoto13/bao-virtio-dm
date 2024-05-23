use event_manager::{EventOps, Events, MutEventSubscriber};
use log::error;
use vmm_sys_util::epoll::EventSet;
use vmm_sys_util::eventfd::EventFd;

use crate::console::virtio::console_handler::ConsoleQueueHandler;
use crate::device::SingleFdSignalQueue;

pub const INPUT_QUEUE_INDEX: u16 = 0;
pub const OUTPUT_QUEUE_INDEX: u16 = 1;

const INPUT_IOEVENT_DATA: u32 = INPUT_QUEUE_INDEX as u32;
const OUTPUT_IOEVENT_DATA: u32 = OUTPUT_QUEUE_INDEX as u32;

// This object simply combines the more generic `ConsoleQueueHandler` with a concrete queue
// signalling implementation based on `EventFd`s, and then also implements `MutEventSubscriber`
// to interact with the event manager. `ioeventfd` is the `EventFd` connected to queue
// notifications coming from the driver.
pub(crate) struct QueueHandler {
    pub inner: ConsoleQueueHandler<SingleFdSignalQueue>,
    pub input_ioeventfd: EventFd,
    pub output_ioeventfd: EventFd,
}

impl QueueHandler {
    // Helper method that receives an error message to be logged and the `ops` handle
    // which is used to unregister all events.
    fn handle_error<S: AsRef<str>>(&self, s: S, ops: &mut EventOps) {
        error!("{}", s.as_ref());
        ops.remove(Events::empty(&self.input_ioeventfd))
            .expect("Failed to remove input ioeventfd");
        ops.remove(Events::empty(&self.output_ioeventfd))
            .expect("Failed to remove output ioeventfd");
    }
}

/// Implement the `MutEventSubscriber` trait for `QueueHandler` to handle the dispatched
/// events (Ioeventfds) from the event manager.
impl MutEventSubscriber for QueueHandler {
    fn process(&mut self, events: Events, ops: &mut EventOps) {
        if events.event_set() != EventSet::IN {
            self.handle_error("Unexpected event_set", ops);
            return;
        }

        match events.data() {
            INPUT_IOEVENT_DATA => {
                if self.input_ioeventfd.read().is_err() {
                    self.handle_error("Input ioeventfd read", ops);
                } else if let Err(e) = self.inner.process_input_queue() {
                    self.handle_error(format!("Process input queue error {:?}", e), ops);
                }
            }
            OUTPUT_IOEVENT_DATA => {
                if self.output_ioeventfd.read().is_err() {
                    self.handle_error("Output ioeventfd read", ops);
                }
                if let Err(e) = self.inner.process_output_queue() {
                    self.handle_error(format!("Process output queue error {:?}", e), ops);
                }
            }
            _ => self.handle_error("Unexpected ioeventfd", ops),
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        ops.add(Events::with_data(
            &self.input_ioeventfd,
            INPUT_IOEVENT_DATA,
            EventSet::IN,
        ))
        .expect("Failed to init input queue handler");

        ops.add(Events::with_data(
            &self.output_ioeventfd,
            OUTPUT_IOEVENT_DATA,
            EventSet::IN,
        ))
        .expect("Failed to init output queue handler");
    }
}
