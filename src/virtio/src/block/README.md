# VirtIO Block

## Overview
The VirtIO Block device is a virtual block device specification designed for efficient and high-performance
block device access within virtualized environments. It enables virtual machines (VMs) to interact with 
block storage devices, such as hard drives and SSDs, through a standardized interface, optimizing performance
and resource utilization. The frontend driver, in the frontend VM, places read, write, and other requests 
onto the virtqueue, so that the backend driver, in the backend VM, can process them accordingly. 
Communication between the frontend and backend is based on the virtio kick and notify mechanism.

## Purpose

The primary purpose of the VirtIO Block device within the context of Bao Hypervisor is to provide 
VMs with access to virtual block storage devices, enhancing its storage virtualization capabilities,
and enabling VMs to efficiently interact with block storage devices for non-volatile 
data storage and retrieval.

## Requirements
- VirtIO Block support on the Frontend VM (e.g. `CONFIG_VIRTIO_BLK` on buildroot)