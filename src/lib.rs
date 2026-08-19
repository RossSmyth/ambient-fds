#![cfg_attr(feature = "nightly", feature(unix_socket_ancillary_data, linux_pidfd))]

use std::num::IntErrorKind;

#[cfg(feature = "checked_api")]
mod checked;
#[cfg(feature = "fd_store")]
pub mod fd_store;
#[cfg(feature = "nightly")]
pub mod notify;
mod raw;

#[cfg(feature = "checked_api")]
pub use checked::*;
pub use raw::*;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EnvVarErrorKind {
    /// The environment variables was not in the environment
    NotFound,
    /// The variable was not unicode encoded
    NotUnicode(Vec<u8>),
    /// The variables was supposed to be a number, but was not.
    NotANumber(IntErrorKind),
    /// Invalid format
    InvalidFormat(String),
}

/// Errors related to reading the environment variables.
#[derive(Debug, Clone)]
pub struct EnvVarError {
    /// The name of the environment variable. Do not attempt to fix or set the
    /// variables yourself, consult systemd documentation.
    pub name: String,
    pub kind: EnvVarErrorKind,
}
