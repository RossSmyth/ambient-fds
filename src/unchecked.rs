use std::{env, os::fd::RawFd};

/// Unprocessed or validated FDs
pub struct RawAmbientFd {
    fd: RawFd,
    name: Box<str>,
}

impl RawAmbientFd {
    /// Get the unerlying FD. It is up to the user to use it correctly.
    pub fn get_fd(&self) -> RawFd {
        self.fd
    }

    /// Get the name. May be empty.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

/// This is a constant in a systemd header, and people hardcode this. So it's probably fine to do
/// the same here.
const SD_LISTEN_FDS_START: i32 = 3;

/// This function doesn't do any checking, just collects.
/// If there isn't anything to collect, returns an empty Vec.
///
/// It is recommended to use the above API has it does checking
/// through systemd's API to ensure the FDs are chill guys, and
/// lets the user know what type of FD they are.
///
/// If there are any errors, for example the environment variables are not
/// formatted correctly, it returns an empty Vec.
///
/// This does not always mean systemd didn't provide something,
/// but can also mean that this process was not run by systemd.
pub fn get_raw_ambient_fds() -> Vec<RawAmbientFd> {
    let Some(raw_fd_count) = env::var_os("LISTEN_FDS") else {
        return Vec::new();
    };
    // This must be UTF-8
    let Some(raw_fd_count) = raw_fd_count.to_str() else {
        return Vec::new();
    };
    let Ok(fd_count) = raw_fd_count.parse() else {
        return Vec::new();
    };

    let Some(raw_names) = env::var_os("LISTEN_FDNAMES") else {
        return Vec::new();
    };
    // I think these have to be UTF-8
    let Some(raw_names) = raw_names.to_str() else {
        return Vec::new();
    };
    let name_list = raw_names.split(":");

    (SD_LISTEN_FDS_START..)
        .skip(1) // The start is not counted as an FD
        .take(fd_count)
        .zip(name_list)
        .map(|(fd, name)| RawAmbientFd {
            fd,
            name: name.into(),
        })
        .collect()
}
