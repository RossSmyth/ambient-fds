use std::{
    env,
    os::{fd::BorrowedFd, unix::net::UnixDatagram},
    time::Duration,
};

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

    fn notify_fds(&mut self, message: &[&str], fds: &[()]) {
        todo!("Nightly")
    }

    fn notify(&mut self, message: &[&str]) {
        self.notify_fds(message, &[])
    }

    fn notify_single(&mut self, message: &str) {
        self.notify(&[message])
    }

    pub fn send_ready(&mut self) {
        self.notify_single(ready());
    }

    pub fn send_reloading(&mut self) {
        self.notify(&[reloading(), &monotonic_usec(todo!("CLOCK_MONOTONIC"))])
    }

    pub fn send_stopping(&mut self) {
        self.notify_single(stopping());
    }

    pub fn send_monotonic(&mut self, timestamp: u128) {
        self.notify_single(&monotonic_usec(timestamp));
    }

    pub fn send_status(&mut self, state: &str) {
        self.notify_single(&status(state))
    }

    /// Corresponds to NOTIFYACCESS=
    /// Essentially who is allowed to send messages to the systemd socket.
    /// This API.
    pub fn send_socket_access(&mut self, state: SocketAccess) {
        self.notify_single(notify_access(state));
    }

    pub fn send_errno(&mut self, error: i32) {
        self.notify_single(&errno(error));
    }

    pub fn send_bus_error(&mut self, error: &str) {
        self.notify_single(&bus_error(error));
    }

    pub fn send_varlink_error(&mut self, error: &str) {
        self.notify_single(&varlink_error(error));
    }

    pub fn send_exit_status(&mut self, status: &str) {
        self.notify_single(&exit_status(status));
    }

    /// Should only be used if sending from the process the PID is referring to.
    pub fn send_mainpid(&mut self, pid: u32) {
        self.notify_single(&mainpid(pid));
    }

    /// Should be used when referring to a child process
    pub fn send_mainpidfd(&mut self, pidfd: ()) {
        self.notify_fds(&[main_pidfd()], &[pidfd]);
        todo!("Actually use PidFd")
    }

    /// Corresponds to WATCHDOG=1
    pub fn send_watchdog_update(&mut self) {
        self.notify_single(watchdog_update());
    }

    /// Corresponds to WATCHDOG=trigger
    pub fn send_watchdog_trigger(&mut self) {
        self.notify_single(watchdog_trigger());
    }

    /// Corresponds to WATCHDOG_USEC
    pub fn send_watchdog_timeout(&mut self, timeout: Duration) {
        self.notify_single(&watchdog_timeout(timeout));
    }

    /// Corresponds to EXTEND_TIMEOUT_USEC
    pub fn send_delay_watchdog_timeout(&mut self, timeout: Duration) {
        self.notify_single(&extend_timeout(timeout));
    }

    pub fn send_reset_restart_counters(&mut self) {
        self.notify_single(restart_reset());
    }

    pub fn send_barrier(&mut self, _: BorrowedFd) {
        self.notify_fds(&[barrier()], &[()]);
        todo!("Use FD")
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
