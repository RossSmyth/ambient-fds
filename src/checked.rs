use std::{
    env,
    ffi::CStr,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
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

    pub fn into_kind(self) -> FdKind {
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
    ///
    /// Files using systemd's `OpenFile` will appear as a fifo FD.
    Fifo(OwnedFd),
    /// IPv4 or IPv6 socket FD
    Berkely(OwnedFd),
    /// Unix domain socket FD
    Unix(OwnedFd),
    /// Posix message queue FD
    MessageQueue(OwnedFd),
    /// Special FD, like those under /prov and /sys
    Special(OwnedFd),
    /// Unable to determine FD type
    Unknown(OwnedFd),
}

impl FdKind {
    // Safety: Must be a valid FD that is owned by this.
    unsafe fn new(fd: RawFd) -> Self {
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };

        // Let systemd do most of the checks as it is more comprehensive
        // and robust.
        if is_fifo(fd.as_fd()) {
            Self::Fifo(fd)
        } else if is_berkely(fd.as_fd()) {
            Self::Berkely(fd)
        } else if is_unix(fd.as_fd()) {
            Self::Unix(fd)
        } else if is_queue(fd.as_fd()) {
            Self::MessageQueue(fd)
        } else if is_special(fd.as_fd()) {
            Self::Special(fd)
        } else {
            Self::Unknown(fd)
        }
    }

    pub fn into_fd(self) -> OwnedFd {
        match self {
            FdKind::Unknown(fd)
            | FdKind::Special(fd)
            | FdKind::MessageQueue(fd)
            | FdKind::Unix(fd)
            | FdKind::Fifo(fd)
            | FdKind::Berkely(fd) => fd,
        }
    }
}

fn is_fifo(fd: BorrowedFd) -> bool {
    systemd::daemon::is_fifo(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

fn is_berkely(fd: BorrowedFd) -> bool {
    systemd::daemon::is_socket_inet(
        fd.as_raw_fd(),
        None,
        None,
        Listening::NoListeningCheck,
        None,
    )
    .is_ok()
}

fn is_unix(fd: BorrowedFd) -> bool {
    systemd::daemon::is_socket_unix(
        fd.as_raw_fd(),
        None,
        Listening::NoListeningCheck,
        Option::<&CStr>::None,
    )
    .is_ok()
}

fn is_queue(fd: BorrowedFd) -> bool {
    systemd::daemon::is_mq(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

fn is_special(fd: BorrowedFd) -> bool {
    systemd::daemon::is_special(fd.as_raw_fd(), Option::<&CStr>::None).is_ok()
}

/// Get the ambient FDs that systemd has provided.
/// Call at the begining of the program. Is purposfully not idempotent.
///
/// Note: This function purposefully clears systemd-managed environment variables.
///
/// The only error this functino returns is the errono returned by systemd. It will
/// panic if the environment was set incorrectly as systemd doesn't.
///
/// # Safety
/// * Nothing else can read the FD related environment variables before this call.
/// * If they are set, the environment variables must have been set solely by systemd
pub unsafe fn get_ambient_fds() -> Result<Vec<AmbientFd>, std::io::Error> {
    let fds = match systemd::daemon::listen_fds(false) {
        Ok(fds) if fds.is_empty() => return Ok(Vec::new()),
        Ok(fds) => fds,
        Err(err) => return Err(err),
    };

    let raw_names = match env::var("LISTEN_FDNAMES") {
        Ok(names) => names,
        Err(err) => panic!(
            "systemd always sets 'LISTEN_FDNAMES' correctly, and it must be correct at this point.\n{err:?}"
        ),
    };
    let names = raw_names.split(":");

    // Do not inherit or try to do anything weird.
    unsafe {
        env::remove_var("LISTEN_FDS");
        env::remove_var("LISTEN_PID");
        env::remove_var("LISTEN_FDNAMES");
    }

    Ok(fds
        .iter()
        .zip(names)
        .map(|(raw_fd, name)| AmbientFd {
            name: match name {
                "connection" => FdName::Connection,
                "stored" => FdName::Stored,
                "unknown" => FdName::Unknown,
                name => FdName::Name(name.into()),
            },
            // Safety: It's a valid FD since we got it from systemd
            fd: unsafe { FdKind::new(raw_fd) },
        })
        .collect())
}
