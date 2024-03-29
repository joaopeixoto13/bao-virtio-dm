use super::packet_handler::VsockPacketHandler;
use crate::device::SingleFdSignalQueue;
use event_manager::{EventOps, Events, MutEventSubscriber};
use vmm_sys_util::epoll::EventSet;
use vmm_sys_util::eventfd::EventFd;

// This object simply combines the more generic `VsockPacketHandler` with a concrete queue
// signalling implementation based on `EventFd`s, and then also implements `MutEventSubscriber`
// to interact with the event manager. `ioeventfd` is the `EventFd` connected to queue
// notifications coming from the driver.
pub(crate) struct QueueHandler {
    pub inner: VsockPacketHandler<SingleFdSignalQueue>,
    pub ioeventfd: Vec<EventFd>,
}

/// Implement the `MutEventSubscriber` trait for `QueueHandler` to handle the dispatched
/// events (Ioeventfds) from the event manager.
impl MutEventSubscriber for QueueHandler {
    fn process(&mut self, events: Events, ops: &mut EventOps) {
        let mut error = true;

        // TODO: Have a look at any potential performance impact caused by these conditionals
        // just to be sure.
        if events.event_set() != EventSet::IN {
            println!("unexpected event_set");
        } else if events.data() as usize >= self.ioeventfd.len() {
            println!("unexpected events data {}", events.data());
        } else if let Err(e) = self.inner.process_queue(events.data() as usize) {
            println!("error processing block queue {:?}", e);
        } else {
            error = false;
        }

        if error {
            ops.remove(events)
                .expect("Failed to remove fd from event handling loop");
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        for (index, ioeventfd) in self.ioeventfd.drain(..).enumerate() {
            ops.add(Events::with_data(&ioeventfd, index as u32, EventSet::IN))
                .expect("Failed to init block queue handler");
        }
    }
}
