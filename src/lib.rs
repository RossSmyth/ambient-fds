mod unchecked;

use std::num::IntErrorKind;

pub use unchecked::*;

#[cfg(feature = "checked_api")]
mod checked;

#[cfg(feature = "checked_api")]
pub use checked::*;

#[cfg(feature = "fd_store")]
pub mod fd_store;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EnvVarErrorKind {
    NotFound,
    NotUnicode(Vec<u8>),
    NotANumber(IntErrorKind),
}

#[derive(Debug, Clone)]
pub struct EnvVarError {
    pub name: String,
    pub kind: EnvVarErrorKind,
}
