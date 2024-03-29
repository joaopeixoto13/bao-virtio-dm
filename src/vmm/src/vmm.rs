use api::error::{Error, Result};
use api::types::VMMConfig;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};

use super::vm::Vm;

/// VMM abstraction.
///
/// # Attributes
///
/// * `fd` - The file descriptor for the VMM (e.g. /dev/bao).
/// * `vms` - The list of VMs.
/// * `vcpus` - The list of vCPUs/threads.
pub struct Vmm {
    fd: i32,
    vms: Mutex<Vec<Arc<Vm>>>,
    vcpus: Mutex<Vec<JoinHandle<()>>>,
}

impl TryFrom<VMMConfig> for Vmm {
    type Error = Error;

    /// Try_from method used to create a VMM from a VMM configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The VMM configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    fn try_from(config: VMMConfig) -> Result<Self> {
        // Open the VMM file descriptor.
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/bao")
            .map_err(|_e| Error::OpenFdFailed("/dev/bao", std::io::Error::last_os_error()))?;

        // Create the VMM.
        let vmm = Vmm {
            fd: fd.as_raw_fd(),
            vms: Mutex::new(Vec::new()),
            vcpus: Mutex::new(Vec::new()),
        };

        // Create all VMs.
        for config in config.devices {
            let vm = Vm::new(vmm.fd, config).unwrap();

            // Add the VM to the VMM list.
            vmm.vms.lock().unwrap().push(Arc::new(vm));
        }

        Ok(vmm)
    }
}

impl Vmm {
    /// Run the VMM.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the operation.
    pub fn run(&self) -> Result<()> {
        for vm in self.vms.lock().unwrap().drain(..) {
            // Create a new vCPU/thread to run the I/O events.
            let vm_io = vm.clone();
            self.vcpus.lock().unwrap().push(
                Builder::new()
                    .name(format!("vm_{}_io", vm_io.id))
                    .spawn(move || {
                        vm_io.run_io().unwrap();
                    })
                    .unwrap(),
            );

            // Create a new vCPU/thread to run the VM event manager.
            let vm_evm = vm.clone();
            self.vcpus.lock().unwrap().push(
                Builder::new()
                    .name(format!("vm_{}_evm", vm_evm.id))
                    .spawn(move || {
                        vm_evm.run_event_manager();
                    })
                    .unwrap(),
            );
        }
        Ok(())
    }
}

impl Drop for Vmm {
    /// Drops all handles from the vcpus vector.
    fn drop(&mut self) {
        // Loops until all handles are popped from the vcpus vector
        while let Some(handle) = self.vcpus.lock().unwrap().pop() {
            // Joins the thread represented by the handle
            handle.join().unwrap();
        }
    }
}
