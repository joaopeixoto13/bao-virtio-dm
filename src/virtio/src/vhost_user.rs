use super::device::SingleFdSignalQueue;
use std::io::Result as IoResult;
use vhost_user_frontend::{VirtioInterrupt, VirtioInterruptType};
use vmm_sys_util::eventfd::EventFd;

impl VirtioInterrupt for SingleFdSignalQueue {
    /// Implementation of the trigger method of the VirtioInterrupt trait for BaoInterrupt.
    ///
    /// # Arguments
    ///
    /// * `_int_type` - The type of the interrupt (Used Buffer or Configuration Change Notification).
    ///
    /// # Return
    ///
    /// * `IoResult<()>` - An IoResult containing Ok(()) on success, or an Error on failure.
    fn trigger(&self, _int_type: VirtioInterruptType) -> IoResult<()> {
        Ok(())
    }

    /// Implementation of the notifier method of the VirtioInterrupt trait for BaoInterrupt.
    ///
    /// # Arguments
    ///
    /// * `_int_type` - The type of the interrupt (Used Buffer or Configuration Change Notification).
    ///
    /// # Return
    ///
    /// * `Option<EventFd>` - An Option containing the EventFd associated with the interrupt.
    fn notifier(&self, _int_type: VirtioInterruptType) -> Option<EventFd> {
        Some(self.irqfd.try_clone().unwrap())
    }
}
