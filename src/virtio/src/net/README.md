# VirtIO Net

## Overview
The VirtIO Net device is a virtual network interface designed for efficient networking within 
virtualized environments. It provides a high-performance, low-latency network connection between 
virtual machines (VMs), facilitating communication and data transfer over a virtualized network.

## Purpose
The purpose of the VirtIO Net device is to enable VMs to communicate with each other and with 
external networks seamlessly, while minimizing overhead and maximizing performance. By emulating 
a network interface that is compatible with the VirtIO standard, the VirtIO Net device allows VMs 
to leverage the full capabilities of modern networking technologies within virtualized environments.

## Requirements
- VirtIO Network support on the Frontend VM (e.g. `CONFIG_VIRTIO_NET` on buildroot)