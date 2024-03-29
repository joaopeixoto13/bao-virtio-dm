# Virtual File System

## Overview
The Virtual File System device  is a virtualized file system designed to provide efficient and secure 
access to volatile file storages within virtualized environments. It allows virtual machines (VMs) to 
interact with host file systems and shared folders using a standardized interface or acting as a gateway
to a remote file system, facilitating seamless file I/O operations between VMs and the host system. 
The device interface is based on the Linux Filesystem in Userspace (FUSE) protocol. The device acts as 
the FUSE file system daemon and the driver acts as the FUSE client mounting the file system. The virtio 
file system device provides the mechanism for transporting FUSE requests, much like */dev/fuse* in a 
traditional FUSE application.

## Purpose

The primary purpose of the VirtIO File System device is to enable VMs to access file storage resources in 
a virtualized environment with optimal performance and minimal overhead. By leveraging the VirtFS protocol, 
VMs can mount host file systems or shared folders as virtual file systems, enabling them to read, write, 
and manage files as if they were accessing local storage.

## Requirements
- Virtual File System support on the Frontend VM (e.g. `CONFIG_VIRTIO_FS` on buildroot)