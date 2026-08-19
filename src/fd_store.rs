//! Send and receive FDs to/from systemd's file descriptor store.
//! <https://systemd.io/FILE_DESCRIPTOR_STORE/>
//!
//! This requires that the service is started with the `FileDescriptorStoreMax` attribute
//! in the unit configuration file to be non-zero.
//!
//! <div class="warning">
//! The systemd API is asynchronous, and does not have a general way
//! to known if an FD was successfully stored or not. This should
//! only be used as a best effort API, and a fallback should always
//! be ready in case of a failure. One example is if the service
//! is started without an FD store.
//! </div>
use std::os::fd::BorrowedFd;

#[cfg(feature = "libsystemd")]
pub use self::libsystemd::*;

#[cfg(feature = "notify")]
pub use self::notify_store::*;

/// Struct holding the data needed to store the
/// FD.
pub struct StoreFd<'store, 'fd> {
    fd: BorrowedFd<'fd>,
    name: &'store str,
}

impl<'store, 'fd> StoreFd<'store, 'fd> {
    pub fn new(fd: BorrowedFd<'fd>, name: &'store str) -> Self {
        StoreFd { fd, name }
    }
}

#[cfg(feature = "notify")]
mod notify_store {
    use super::*;
    use crate::notify;

    /// Stores an FD in systemd's FD store.
    ///
    /// This API requires a name, because it is much more useful that way.
    pub fn store_fd_socket<'store, 'fd>(
        socket: &mut notify::SysDSocket,
        StoreFd { fd, name }: StoreFd<'store, 'fd>,
    ) {
        socket.notify_fds(&[notify::fd_store(), notify::fd_name(name).as_str()], &[fd])
    }

    /// Removes an FD from systemd's FD store.
    ///
    /// The FD is not required, just provide the name.
    pub fn socket_remove_fd(socket: &mut notify::SysDSocket, name: &str) {
        socket.notify(&[notify::fd_name(name).as_str(), notify::fd_store_remove()]);
    }
}

#[cfg(feature = "libsystemd")]
mod libsystemd {
    use std::os::fd::AsRawFd;

    use systemd::{
        Error,
        daemon::{STATE_FDNAME, STATE_FDSTORE, STATE_FDSTOREREMOVE, pid_notify_with_fds},
    };

    use super::*;

    /// Stores an FD in systemd's FD store.
    ///
    /// This API requires a name, because it is much more useful that way.
    ///
    /// Stored FDs can provide seamless service restarts, as FDs can be loaded into the store
    /// then when restarted they are obtained again. This can survive even system updates if
    /// done via kexec.
    pub fn store_fd<'store, 'fd>(StoreFd { fd, name }: StoreFd<'store, 'fd>) -> Result<(), Error> {
        let output = pid_notify_with_fds(
            std::process::id()
                .try_into()
                .expect("Process PID exceeds pid_t value range."),
            false,
            [&(STATE_FDSTORE, "1"), &(STATE_FDNAME, name)].into_iter(),
            &[fd.as_raw_fd()],
        );

        match output {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Removes an FD from systemd's FD store.
    ///
    /// The FD is not required, just provide the name.
    ///
    /// For example if the service puts all connections in the FD store
    /// for robustness against service restarts, the connection should
    /// then be removed once the service is done listening to the connection.
    pub fn remove_fd(name: &str) -> Result<(), Error> {
        let output = pid_notify_with_fds(
            std::process::id()
                .try_into()
                .expect("Process PID exceeds pid_t value range."),
            false,
            [&(STATE_FDSTOREREMOVE, "1"), &(STATE_FDNAME, name)].into_iter(),
            &[],
        );

        match output {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
