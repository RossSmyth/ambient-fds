use std::{
    env::{self, VarError},
    os::{fd::RawFd, unix::ffi::OsStringExt},
};

use crate::{EnvVarError, EnvVarErrorKind};

/// Unprocessed or validated FDs
pub struct RawAmbientFd {
    fd: RawFd,
    name: String,
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
const SD_LISTEN_FDS_START: usize = 3;

/// This function doesn't do any checking, just collects.
/// If there isn't anything to collect, returns an empty Vec.
///
/// It is recommended to use the above API has it does checking
/// through systemd's API to ensure the FDs are chill guys, and
/// lets the user know what type of FD they are.
///
/// Systemd docs recommends, essentially, to ignore the exact error
/// and just go to a fallback (if possible) if this fails.
pub fn get_raw_ambient_fds() -> Result<Vec<RawAmbientFd>, EnvVarError> {
    let raw_fd_count = match env::var("LISTEN_FDS") {
        Ok(raw_fd_count) => raw_fd_count,
        Err(VarError::NotPresent) => {
            return Err(crate::EnvVarError {
                name: "LISTEN_FDS".to_string(),
                kind: EnvVarErrorKind::NotFound,
            });
        }
        Err(VarError::NotUnicode(str)) => {
            return Err(EnvVarError {
                name: "LISTEN_FDS".to_string(),
                kind: EnvVarErrorKind::NotUnicode(str.into_vec()),
            });
        }
    };

    let fd_count: usize = match raw_fd_count.parse() {
        Ok(fd_count) => fd_count,
        Err(err) => {
            return Err(EnvVarError {
                name: "LISTEN_FDS".to_string(),
                kind: EnvVarErrorKind::NotANumber(*err.kind()),
            });
        }
    };

    let total_fds = SD_LISTEN_FDS_START + 1 - fd_count;

    // It is C brained so we must check.
    match TryInto::<i32>::try_into(total_fds) {
        Ok(_) => (),
        Err(_) => {
            return Err(EnvVarError {
                name: "LISTEN_FDS".to_string(),
                kind: EnvVarErrorKind::InvalidFormat(format!(
                    "Total number of FDs exceed to possible total."
                )),
            });
        }
    };

    let raw_names = match env::var("LISTEN_FDNAMES") {
        Ok(raw_names) => raw_names,
        Err(VarError::NotPresent) => {
            return Err(crate::EnvVarError {
                name: "LISTEN_FDNAMES".to_string(),
                kind: EnvVarErrorKind::NotFound,
            });
        }
        Err(VarError::NotUnicode(str)) => {
            return Err(EnvVarError {
                name: "LISTEN_FDNAMES".to_string(),
                kind: EnvVarErrorKind::NotUnicode(str.into_vec()),
            });
        }
    };

    let name_list: Vec<String> = raw_names.split(":").map(str::to_string).collect();

    if name_list.len() != total_fds {
        return Err(EnvVarError {
            name: "LISTEN_FDS, LISTEN_FDNAMES".to_string(),
            kind: EnvVarErrorKind::InvalidFormat(format!(
                "Number of FDs specified ({total_fds}), and number of names ({}) mismatch",
                name_list.len()
            )),
        });
    }

    Ok((SD_LISTEN_FDS_START..)
        .skip(1) // The start is not counted as an FD
        .take(fd_count)
        .zip(name_list.into_iter())
        .map(|(fd, name)| RawAmbientFd {
            fd: fd.try_into().unwrap(),
            name: name,
        })
        .collect())
}
