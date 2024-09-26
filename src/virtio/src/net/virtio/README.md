# Quick start

Follow these steps to quickly set up and run a VM with virtio-net capabilities within Bao Hypervisor.

1. **Prepare the configuration file**: Create a configuration file (e.g. *config-virtio-net.yaml*) specifying
the settings for the virtio-net device. One example of a configuration file could be:

```
devices:
    # --- VirtIO Common ---
  - id: 0
    type: "net"
    mmio_addr: 0xa003e00
    data_plane: virtio
    # --- Virtio Net Specific ---
    tap_name: "tap0"
    mac_addr: "52:54:00:12:34:56"
    # -----------------------------
```

2. **Create a TAP device**: A TAP (Network TAP) device is a virtual network interface in Linux that operates at the data link layer 
(Layer 2) of the OSI model. It simulates an Ethernet network interface and allows user-space applications 
to interact with it to transmit and receive Ethernet frames.

Create a TAP device (e.g. `tap0`) with the mode set to `tap`:
```
ip tuntap add dev tap0 mode tap
```

(Note that the TAP device name should be the same as the tap name in the device model configuration file)

Assign one IP address (e.g. `192.168.42.14`) to the tap interface and bring it up. 
```
ifconfig tap0 192.168.42.14 up
```

3. **Create a bridge**: To enable communication between the TAP device (e.g. `tap0`) and the physical network interface (e.g. `eth0`), 
you can create a bridge. A bridge is a virtual network device that connects multiple network interfaces at the 
data link layer (Layer 2) and forwards packets between them.

Create a bridge interface (e.g. `br0`):
```
brctl addbr br0
```

Assign one IP address (e.g. `192.168.42.13`) to the bridge.
```
ifconfig br0 192.168.42.13 netmask 255.255.255.0
```

Add the TAP device (`tap0`) and the NIC (`eth0`) to the bridge:
```
brctl addif br0 eth0
brctl addif br0 tap0
```

In this context, when the virtio-net driver wants to transmit one packet, the device model forwards the data received 
from the frontend to the TAP device, then from the TAP device to the bridge, and finally from the bridge to the physical 
NIC driver, and vice versa for returning data from the NIC to the frontend.

Once the TAP device and the bridge are brought up, you can send and receive network traffic.
You can verify the configuration by running:
```
ifconfig
```

4. **Launch the device model with the virtio-net device**: To launch the device model in the background type:

```
nohup bao-virtio-dm --config /PATH/TO/YOUR/config-virtio-net.yaml > /etc/bao-virtio-dm.log 2>&1 &
```

In the **Frontend VM**, you can use the command `ethtool` to verify the virtual network interface (e.g. `eth0`) configuration:
```
ethtool -i eth0
```

(Note: You should visualize the virtio-net driver and the MMIO address that the driver is operating on)

---

## Test cases

You can validate and test the virtio-net connection by performing the following tests:

1. Using **ping** command:

From **any device connected to the network**, type:
```
ping <FRONTEND_VM_IP_ADDR>
```

From the **Frontend VM**:
```
ping <TARGET_IP_ADDR>
```

Additionally, to verify if the Frontend VM has access to the external world, you can simply:
```
ping 8.8.8.8
```

2. Using **wget** command

```
wget http://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.16.11.tar.xz
```

3. Using **iperf3** command to **measure inter-VM or external device connection bandwidth** 

You can use the `iperf3` command to measure the total transmit and receive bandwidth between VMs or between a VM and an external device.

**Transmit bandwidth**: To measure the transmit bandwidth (guest to host transmit path), 
you must run the server on your host and the client on the guest/frontend VM. Note that the host here can be any external device connceted to the network, or even other VM with a network (virtual or physical) interface.

From the **host**, create the server:
```
iperf3 -s
```

From the **Frontend VM**, create the client:
```
iperf3 -c <HOST_IP_ADDR>
```

**Receive bandwidth**: To measure the receive bandwidth (host to guest receive path), 
you must run the server on the guest/frontend VM and the client on your host.

From the **Frontend VM**, create the server:
```
iperf3 -s
```

From the **host**, create the client:
```
iperf3 -c <FRONTEND_VM_IP_ADDR>
```

4. Establish a **ssh connection**

You can establish a `ssh` connection from/to any device connected to the network.

From  **any device connected to the network**, you can connect to the Frontend VM by:
```
ssh root@<FRONTEND_VM_IP_ADDR>
```

Or if you want to establish a conncetion from the **Frontend VM**:
```
ssh <TARGET_IP_ADDR>
```