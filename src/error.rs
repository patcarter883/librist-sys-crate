//! Error handling for librist-sys
//!
//! This module provides error types for the librist FFI bindings.
//! Works in both std and no_std environments.

use core::ffi::c_int;
use core::fmt;

/// Result type alias using librist-sys Error
pub type Result<T> = core::result::Result<T, Error>;

/// Error types for librist operations.
///
/// This enum covers the main error categories that can occur when
/// working with the librist library.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// Internal error from the librist library (contains the error code)
    Internal(i32),
    /// A null pointer was provided where a valid pointer was required
    NullPointer,
    /// An invalid argument was provided
    InvalidArgument,
    /// Memory allocation failed
    AllocationFailed,
    /// Failed to parse a RIST URL or address
    ParseError,
    /// A connection error occurred
    ConnectionError,
    /// Operation timed out
    Timeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Internal(code) => write!(f, "RIST error code: {}", code),
            Error::NullPointer => write!(f, "Null pointer provided"),
            Error::InvalidArgument => write!(f, "Invalid argument"),
            Error::AllocationFailed => write!(f, "Memory allocation failed"),
            Error::ParseError => write!(f, "Parse error"),
            Error::ConnectionError => write!(f, "Connection error"),
            Error::Timeout => write!(f, "Operation timed out"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Internal(_) => "Internal error",
            Error::NullPointer => "Null pointer",
            Error::InvalidArgument => "Invalid argument",
            Error::AllocationFailed => "Allocation failed",
            Error::ParseError => "Parse error",
            Error::ConnectionError => "Connection error",
            Error::Timeout => "Timeout",
        }
    }
}

impl From<c_int> for Error {
    fn from(code: c_int) -> Self {
        // Map common error codes to specific variants
        // Based on librist error codes in librist.h
        match code {
            -1 => Error::AllocationFailed,
            -2 => Error::NullPointer,
            -3 | -4 => Error::InvalidArgument,
            -5 => Error::ParseError,
            -6 => Error::ConnectionError,
            -7 => Error::Timeout,
            _ => Error::Internal(code),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::ffi::NulError> for Error {
    fn from(_: std::ffi::NulError) -> Self {
        Error::InvalidArgument
    }
}

/// Map a librist return code to a Result.
///
/// Returns `Ok(())` for success (0), or the appropriate Error for failure.
#[inline]
pub fn to_result(code: c_int) -> Result<c_int> {
    if code < 0 {
        Err(Error::from(code))
    } else {
        Ok(code)
    }
}

/// Check for null pointer and convert to Result.
#[inline]
pub fn null_check<T>(ptr: *mut T) -> Result<*mut T> {
    if ptr.is_null() {
        Err(Error::NullPointer)
    } else {
        Ok(ptr)
    }
}
