use std::{
    env,
    io::IoSlice,
    os::{
        fd::{AsFd, BorrowedFd, RawFd},
        linux::process::PidFd,
        unix::net::{SocketAncillary, UnixDatagram},
    },
    slice,
    time::Duration,
};

use nix::time::ClockId;

#[derive(Debug)]
pub struct SysDSocket(UnixDatagram);

impl SysDSocket {
    /// Connect to the systemd notification socket
    /// Should probably be the first function called.
    ///
    /// # Safety
    /// * This is the only time this socket FD is read, and nothing else (systemd included) holds the socket FD from this point forward.
    pub unsafe fn open() -> Option<Self> {
        let sock = env::var_os("NOTIFY_SOCKET")?;

        // Just panic because if these doesn't work it means something is very wrong.
        let connection = UnixDatagram::unbound().unwrap();
        connection.connect(sock).unwrap();

        Some(Self(connection))
    }

    fn notify_fds(&self, message: &[&str], fds: &[BorrowedFd]) {
        let mut multiple = 2;
        let mut buf = vec![];

        let mut data = loop {
            buf = vec![0; multiple * size_of_val(fds)];

            let mut data = SocketAncillary::new(&mut buf);

            if data.add_fds(to_raw_fds(fds)) {
                break data;
            } else {
                multiple = multiple + 1;
            }
        };

        let io: Vec<_> = message
            .into_iter()
            .map(|&str| IoSlice::new(str.as_bytes()))
            .collect();

        let _ = self.0.send_vectored_with_ancillary(&io, &mut data);
    }

    fn notify(&self, message: &[&str]) {
        self.notify_fds(message, &mut [])
    }

    fn notify_single(&self, message: &str) {
        self.notify(&[message])
    }

    pub fn send_ready(&self) {
        self.notify_single(ready());
    }

    pub fn send_reloading(&self) {
        let time: Duration = nix::time::clock_gettime(ClockId::CLOCK_MONOTONIC)
            .unwrap()
            .into();

        self.notify(&[reloading(), &monotonic_usec(time.as_micros())])
    }

    pub fn send_stopping(&self) {
        self.notify_single(stopping());
    }

    pub fn send_monotonic(&self, timestamp: u128) {
        self.notify_single(&monotonic_usec(timestamp));
    }

    pub fn send_status(&self, state: &str) {
        self.notify_single(&status(state))
    }

    /// Corresponds to NOTIFYACCESS=
    /// Essentially who is allowed to send messages to the systemd socket.
    /// This API.
    pub fn send_socket_access(&self, state: SocketAccess) {
        self.notify_single(notify_access(state));
    }

    pub fn send_errno(&self, error: i32) {
        self.notify_single(&errno(error));
    }

    pub fn send_bus_error(&self, error: &str) {
        self.notify_single(&bus_error(error));
    }

    pub fn send_varlink_error(&self, error: &str) {
        self.notify_single(&varlink_error(error));
    }

    pub fn send_exit_status(&self, status: &str) {
        self.notify_single(&exit_status(status));
    }

    /// Should only be used if sending from the process the PID is referring to.
    pub fn send_mainpid(&self, pid: u32) {
        self.notify_single(&mainpid(pid));
    }

    /// Should be used when referring to a child process
    pub fn send_mainpidfd(&self, pidfd: &PidFd) {
        self.notify_fds(&[main_pidfd()], &mut [pidfd.as_fd()]);
    }

    /// Corresponds to WATCHDOG=1
    pub fn send_watchdog_update(&self) {
        self.notify_single(watchdog_update());
    }

    /// Corresponds to WATCHDOG=trigger
    pub fn send_watchdog_trigger(&self) {
        self.notify_single(watchdog_trigger());
    }

    /// Corresponds to WATCHDOG_USEC
    pub fn send_watchdog_timeout(&self, timeout: Duration) {
        self.notify_single(&watchdog_timeout(timeout));
    }

    /// Corresponds to EXTEND_TIMEOUT_USEC
    pub fn send_delay_watchdog_timeout(&self, timeout: Duration) {
        self.notify_single(&extend_timeout(timeout));
    }

    pub fn send_reset_restart_counters(&self) {
        self.notify_single(restart_reset());
    }

    pub fn send_barrier(&self, fd: BorrowedFd) {
        self.notify_fds(&[barrier()], &mut [fd]);
    }
}

fn ready() -> &'static str {
    "READY=1"
}

fn reloading() -> &'static str {
    "RELOADING=1"
}

fn stopping() -> &'static str {
    "STOPPING=1"
}

fn monotonic_usec(timestamp: u128) -> String {
    format!("MONOTONIC_USEC={timestamp}")
}

fn status(status: &str) -> String {
    format!("STATUS={status}")
}

/// Controls what processes can send systemd messages
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SocketAccess {
    /// Do not accept any status updates from the service
    None,
    /// Only accept status updates from the main process ([`SysDSocket::send_mainpid`])
    Main,
    /// Only accept status updates from the main process, control processes, or from the `Exec*` unit commands.
    Exec,
    /// Accept status updates from all service processes
    All,
}

fn notify_access(state: SocketAccess) -> &'static str {
    match state {
        SocketAccess::None => "NOTIFYACCESS=none",
        SocketAccess::Main => "NOTIFYACCESS=main",
        SocketAccess::Exec => "NOTIFYACCESS=exec",
        SocketAccess::All => "NOTIFYACCESS=all",
    }
}

fn errno(errno: i32) -> String {
    format!("ERRNO={errno}")
}

fn bus_error(error: &str) -> String {
    format!("BUSERROR={error}")
}

fn varlink_error(error: &str) -> String {
    format!("VARLINKERROR={error}")
}

fn exit_status(status: &str) -> String {
    format!("EXIT_STATUS={status}")
}

fn mainpid(pid: u32) -> String {
    format!("MAINPID={pid}")
}

fn main_pidfd() -> &'static str {
    "MAINPIDFD=1"
}

fn watchdog_update() -> &'static str {
    "WATCHDOG=1"
}

fn watchdog_trigger() -> &'static str {
    "WATCHDOG=trigger"
}

fn watchdog_timeout(timeout: Duration) -> String {
    format!("WATCHDOG_USEC={}", timeout.as_micros())
}

fn extend_timeout(timeout: Duration) -> String {
    format!("WATCHDOG_USEC={}", timeout.as_micros())
}

fn restart_reset() -> &'static str {
    "RESTART_RESET=1"
}

fn barrier() -> &'static str {
    "BARRIER=1"
}

fn to_raw_fds<'a>(fds: &'a [BorrowedFd<'a>]) -> &'a [RawFd] {
    // Safety: BorrowedFd is transparent over RawFd
    unsafe { slice::from_raw_parts(fds.as_ptr() as _, fds.len()) }
}
