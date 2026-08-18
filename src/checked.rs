use nix::sys::stat::{SFlag, fstat};
use std::{
    env,
    ffi::CStr,
    os::fd::{AsRawFd, BorrowedFd, RawFd},
};
use systemd::daemon::Listening;

/// Represents an ambient FD that systemd has provided.
///
/// Attempts to get the name of the FD if one has been provided,
/// and checks the type of FD.
#[derive(Debug)]
pub struct AmbientFd {
    name: FdName,
    fd: FdKind,
}

impl AmbientFd {
    pub fn get_fd(&self) -> &FdKind {
        &self.fd
    }

    pub fn get_name(&self) -> &FdName {
        &self.name
    }

    pub fn into_fd(self) -> FdKind {
        self.fd
    }
}

/// Represents the name and context of what the name means.
#[derive(Debug)]
pub enum FdName {
    // Name of the FD systemd provided.
    //
    // This name is either provided via [`OpenFile`](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html#OpenFile=)
    // ```ini
    // # ...
    // OpenFile = "/mnt/config.txt:config:read-only"
    // # ...
    // ```
    // If the above unit file is used, then the name will be `config`, and opened with read-only permissions.
    //
    // The name can also be set with the systemd [File Descriptor Store](https://systemd.io/FILE_DESCRIPTOR_STORE/).
    //
    // Without a name, it is often difficult to determine what the FD is, thus it is highly recommended to set.
    Name(Box<str>),
    /// Was in systemd's FD store, but no name was provided.
    ///
    /// This can persist FDs across service restarts, kexec events, and soft-reboots.
    Stored,
    /// This service has been activated via `Accept=yes` in a systemd socket file
    /// This is the connection.
    Connection,
    /// No name was received for this FD, was not in the FD store, and was not a socket-activated connection.
    Unknown,
}

/// Types of FDs received
#[derive(Debug)]
pub enum FdKind {
    /// SystemV FIFO FD
    Fifo(BorrowedFd<'static>),
    /// IPv4 or IPv6 socket FD
    Berkely(BorrowedFd<'static>),
    /// Unix domain socket FD
    Unix(BorrowedFd<'static>),
    /// Posix message queue FD
    MessageQueue(BorrowedFd<'static>),
    /// Normal file FD
    File(BorrowedFd<'static>),
    /// Special FD, like those under /prov and /sys
    Special(BorrowedFd<'static>),
    /// Unable to determine FD type
    Unknown(BorrowedFd<'static>),
}

impl FdKind {
    // Safety: Must be a valid FD that is around for the whole process
    unsafe fn new(fd: RawFd) -> Self {
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };

        // Let systemd do most of the checks as it is more comprehensive
        // and robust.
        if is_fifo(fd) {
            Self::Fifo(fd)
        } else if is_berkely(fd) {
            Self::Berkely(fd)
        } else if is_unix(fd) {
            Self::Unix(fd)
        } else if is_queue(fd) {
            Self::MessageQueue(fd)
        } else if is_special(fd) {
            Self::Special(fd)
        } else if let Ok(mode) = fstat(fd)
            && SFlag::from_bits_truncate(mode.st_mode)
                .contains(SFlag::S_IFDIR | SFlag::S_IFREG | SFlag::S_IFLNK)
        {
            // We shall consider a File to be a symlink, directory, or regular file
            Self::File(fd)
        } else {
            Self::Unknown(fd)
        }
    }
}

fn is_fifo(fd: BorrowedFd<'static>) -> bool {
    systemd::daemon::is_fifo(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

fn is_berkely(fd: BorrowedFd<'static>) -> bool {
    systemd::daemon::is_socket_inet(
        fd.as_raw_fd(),
        None,
        None,
        Listening::NoListeningCheck,
        None,
    )
    .is_ok()
}

fn is_unix(fd: BorrowedFd<'static>) -> bool {
    systemd::daemon::is_socket_unix(
        fd.as_raw_fd(),
        None,
        Listening::NoListeningCheck,
        Option::<&CStr>::None,
    )
    .is_ok()
}

fn is_queue(fd: BorrowedFd<'static>) -> bool {
    systemd::daemon::is_mq(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

fn is_special(fd: BorrowedFd<'static>) -> bool {
    systemd::daemon::is_special(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

/// Get the ambient FDs that systemd has provided.
/// Call at the begining of the program. Is purposfully not idempotent.
///
/// Note: This function purposefully clears systemd-managed environment variables.
///
/// Safety:
/// * Nothing else can write to environment variables before this function is called.
/// * systemd must be the provider of the FDs
pub unsafe fn get_ambient_fds() -> Vec<AmbientFd> {
    let fds = match systemd::daemon::listen_fds(false) {
        Ok(fds) if fds.len() == 0 => return Vec::new(),
        Ok(fds) => fds,
        Err(err) => panic!("Unable to get FDs:\n{:?}", err),
    };

    let raw_names = match env::var("LISTEN_FDNAMES") {
        Ok(names) => names,
        _ => panic!("Unable to get FD names, systemd should always set this."),
    };
    let names = raw_names.split(":");

    // Do not inherit or try to do anything weird.
    unsafe {
        env::remove_var("LISTEN_FDS");
        env::remove_var("LISTEN_PID");
        env::remove_var("LISTEN_FDNAMES");
    }

    fds.iter()
        .zip(names.into_iter())
        .map(|(raw_fd, name)| AmbientFd {
            name: match name {
                "connection" => FdName::Connection,
                "stored" => FdName::Stored,
                "unknown" => FdName::Unknown,
                name => FdName::Name(name.into()),
            },
            // Safety: It's a valid FD since we got it from systemd
            fd: unsafe { FdKind::new(raw_fd.into()) },
        })
        .collect()
}
