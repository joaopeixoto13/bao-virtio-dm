# Virtual Socket

## Overview
The VirtIO socket (vsock) device is a virtual device that facilitates communication between virtual 
machines (VMs). It provides a high-performance, efficient, and secure communication channel for 
inter-VM communication within virtualized environments. The application running in the guest communicates over VM sockets i.e over AF_VSOCK sockets. The application running on the host connects to a unix socket on the host i.e communicates over AF_UNIX sockets.
Because the virtual sockets do not rely on 
the host’s networking stack at all, it can be used with VMs that have no network interfaces 
(Ethernet or TCP/IP stack), reducing the attack surface.

## Purpose

The primary purpose of the VirtIO socket device is to enable communication between virtual machines
running on the same host or across different hosts within a virtualized environment. 
It allows VMs to exchange data, messages, and other forms of communication securely and efficiently.

## Requirements
- VirtIO vsockets support on the Frontend VM (e.g. `CONFIG_VIRTIO_VSOCKETS` on buildroot)