use std::os::fd::{AsRawFd, BorrowedFd};

use systemd::{
    Error,
    daemon::{STATE_FDNAME, STATE_FDSTORE, STATE_FDSTOREREMOVE, pid_notify_with_fds},
};

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

/// Stores an FD in systemd's FD store
/// https://systemd.io/FILE_DESCRIPTOR_STORE/
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
