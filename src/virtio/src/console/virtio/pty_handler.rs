use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, Mutex};

use api::types::DeviceConfig;
use event_manager::{EventOps, Events, MutEventSubscriber};
use libc::IN_NONBLOCK;
use std::os::unix::net::UnixStream;
use virtio_console::console::Console;
use vm_memory::WriteVolatile;
use vmm_sys_util::epoll::EventSet;
use vmm_sys_util::eventfd::EventFd;

const SOURCE_PTY: u32 = 0;
const SOURCE_SOCKET: u32 = 1;

const BUFFER_SIZE: usize = 128;

pub(super) struct PtyHandler<W: Write + WriteVolatile> {
    pub pty: File,
    pub socket: UnixStream,
    pub console: Arc<Mutex<Console<W>>>,
    pub input_ioeventfd: EventFd,
}

impl<W> PtyHandler<W>
where
    W: Write + WriteVolatile,
{
    pub fn new(
        socket: UnixStream,
        console: Arc<Mutex<Console<W>>>,
        input_ioeventfd: EventFd,
        config: &DeviceConfig,
    ) -> Self {
        let pty = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(IN_NONBLOCK)
            .open("/dev/ptmx")
            .unwrap();

        let pty_name = unsafe {
            libc::grantpt(pty.as_raw_fd());
            libc::unlockpt(pty.as_raw_fd());
            std::ffi::CStr::from_ptr(libc::ptsname(pty.as_raw_fd()))
        };

        let pty_path = if let Some(pty_alias) = config.pty_alias.clone() {
            std::os::unix::fs::symlink(pty_name.to_str().unwrap(), pty_alias.as_str())
                .expect(&format!("Failed to create pty handler alias {}", pty_alias));
            pty_alias
        } else {
            String::from(pty_name.to_str().unwrap())
        };

        println!("virtio-console device id {} at {}", config.id, pty_path);

        Self {
            pty,
            socket,
            console,
            input_ioeventfd,
        }
    }
}

impl<W> MutEventSubscriber for PtyHandler<W>
where
    W: Write + WriteVolatile,
{
    fn init(&mut self, ops: &mut EventOps) {
        ops.add(Events::with_data(
            &self.pty,
            SOURCE_PTY,
            EventSet::IN | EventSet::EDGE_TRIGGERED,
        ))
        .expect("Failed to init pty event");

        ops.add(Events::with_data(
            &self.socket,
            SOURCE_SOCKET,
            EventSet::IN | EventSet::EDGE_TRIGGERED,
        ))
        .expect("Failed to init socket event");
    }

    fn process(&mut self, events: Events, ops: &mut EventOps) {
        let mut buf = [0u8; BUFFER_SIZE];

        match events.data() {
            SOURCE_PTY => {
                while let Ok(n) = self.pty.read(&mut buf) {
                    let mut v: Vec<_> = buf[..n].iter().cloned().collect();
                    self.console.lock().unwrap().enqueue_data(&mut v).unwrap();
                    self.input_ioeventfd.write(1).unwrap();
                }
            }
            SOURCE_SOCKET => {
                while let Ok(n) = self.socket.read(&mut buf) {
                    let v: Vec<_> = buf[..n].iter().cloned().collect();
                    self.pty.write(&v).unwrap();
                }
            }
            _ => {
                log::error!(
                    "PtyHandler unexpected event data: {}. Removing event...",
                    events.data()
                );
                ops.remove(events).expect("Failed to remove event");
            }
        }
    }
}
