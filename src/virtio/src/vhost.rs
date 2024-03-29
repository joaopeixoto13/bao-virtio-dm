use api::error::Result;
use std::sync::Arc;
use vhost::VhostUserMemoryRegionInfo;
use vhost_user_frontend::GuestMemoryMmap;
use vm_memory::{GuestAddressSpace, GuestMemory};

pub const VHOST_FEATURES: u64 = 0x13D000000;

pub struct VhostKernelCommon {
    pub features: u64,
}

impl VhostKernelCommon {
    pub fn new(features: u64) -> Result<Self> {
        Ok(VhostKernelCommon { features })
    }

    pub fn features(&self) -> u64 {
        self.features
    }

    pub fn memory(&self, mem: &Arc<GuestMemoryMmap>) -> Result<Vec<VhostUserMemoryRegionInfo>> {
        let mut regions = Vec::new();
        let mem_region: &GuestMemoryMmap = &mem.memory();
        for region in mem_region.iter() {
            let region = match VhostUserMemoryRegionInfo::from_guest_region(region) {
                Ok(region) => region,
                Err(e) => {
                    println!("Failed to create memory region: {:?}", e);
                    panic!("Failed to create memory region: {:?}", e);
                }
            };
            regions.push(region);
        }

        if regions.is_empty() {
            println!("No memory regions found");
            panic!("No memory regions found");
        }

        Ok(regions)
    }
}
