use std::fs::{read_dir, read_link, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
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
    pub pty_path: String,
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
            pty_path,
            socket,
            console,
            input_ioeventfd,
        }
    }

    /// Check if the PTY is currently open by any process (e.g., picocom / minicom)
    fn is_opened(&self) -> std::io::Result<bool> {
        let pty_path = Path::new(self.pty_path.as_str());
        let proc_dir = Path::new("/proc");

        // Iterate through the process directories in /proc
        for entry in read_dir(proc_dir)? {
            let entry = entry?;
            let pid_dir = entry.path();

            // Skip entries that are not directories or are not numeric (non-process directories)
            if !pid_dir.is_dir() {
                continue;
            }

            if let Some(pid) = pid_dir.file_name().and_then(|name| name.to_str()) {
                if pid.chars().all(|c| c.is_digit(10)) {
                    // Construct the path to the fd directory
                    let fd_dir = pid_dir.join("fd");

                    // Iterate through file descriptors
                    if let Ok(fds) = read_dir(fd_dir) {
                        for fd in fds {
                            if let Ok(fd) = fd {
                                // Resolve the symlink to get the actual file path
                                if let Ok(link_path) = read_link(fd.path()) {
                                    if link_path == pty_path {
                                        return Ok(true); // PTY is open
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(false) // No process has opened the PTY
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
                    // TODO: We should understand why the SOURCE_PTY event is not triggered if the backend console is opened.
                    // (As the `self.pty.write(&v).unwrap();` line within the `SOURCE_SOCKET` event is always executed upon receiving data from the frontend console
                    // that needs to be written to the backend console, this event should be triggered regardless of the backend console being opened or not.)
                    // In such cases, we must not enqueue the frontend guest console data (output queue) back to the frontend console (receive queue).
                    if self.is_opened().unwrap().eq(&true) {
                        self.console.lock().unwrap().enqueue_data(&mut v).unwrap();
                        self.input_ioeventfd.write(1).unwrap();
                    }
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
