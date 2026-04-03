//! FFI bindings for librist - Reliable Internet Stream Transport protocol library
//!
//! This crate provides low-level FFI bindings to the librist C library.
//! It supports both `std` and `no_std` environments.
//!
//! # Features
//!
//! - `std` (default): Enables standard library support and thiserror-based error types
//! - `no_std`: Enables embedded/no-std environments
//! - `safe-wrappers`: Provides safe wrapper functions (requires `std`)
//!
//! # Usage
//!
//! ```rust
//! use librist_sys::{rist_profile, rist_log_level};
//! ```
//!
//! # Safety
//!
//! These bindings involve raw FFI calls to a C library. All FFI functions
//! are `unsafe` and require careful handling. The `safe-wrappers` feature
//! provides safer alternatives that perform validation.

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = concat!("librist-sys v", env!("CARGO_PKG_VERSION"))]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// ============================================================================
// Conditional Imports
// ============================================================================

#[cfg(feature = "no_std")]
use core::ffi::{c_char, c_int, c_void};

#[cfg(not(feature = "no_std"))]
use std::os::raw::{c_char, c_int, c_void};

// ============================================================================
// Error Handling Module (std only)
// ============================================================================

#[cfg(feature = "std")]
pub mod error;

#[cfg(feature = "std")]
pub use error::{Error, Result};

// ============================================================================
// Safe Wrappers Module (std + safe-wrappers only)
// ============================================================================

#[cfg(all(feature = "std", feature = "safe-wrappers"))]
pub mod safe;

#[cfg(all(feature = "std", feature = "safe-wrappers"))]
pub use safe::*;

// ============================================================================
// Manual FFI Bindings
// ============================================================================

mod bindings;
pub use bindings::*;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that opaque types are zero-sized (they are only used behind
    /// pointers and must never be instantiated by Rust code).
    #[test]
    fn opaque_types_are_zero_sized() {
        assert_eq!(core::mem::size_of::<rist_ctx>(), 0);
        assert_eq!(core::mem::size_of::<rist_peer>(), 0);
        assert_eq!(core::mem::size_of::<rist_oob_block>(), 0);
    }

    /// The stats union must be at least as large as both variants.
    #[test]
    fn stats_union_size() {
        let union_size = core::mem::size_of::<rist_stats_union>();
        assert!(union_size >= core::mem::size_of::<rist_stats_sender_peer>());
        assert!(union_size >= core::mem::size_of::<rist_stats_receiver_flow>());
    }

    /// Sanity-check that enum discriminants match the expected C values.
    #[test]
    fn enum_discriminants() {
        assert_eq!(rist_profile::RIST_PROFILE_SIMPLE as c_int, 0);
        assert_eq!(rist_profile::RIST_PROFILE_MAIN as c_int, 1);
        assert_eq!(rist_profile::RIST_PROFILE_ADVANCED as c_int, 2);

        assert_eq!(rist_log_level::RIST_LOG_DISABLE as c_int, -1);
        assert_eq!(rist_log_level::RIST_LOG_ERROR as c_int, 3);
        assert_eq!(rist_log_level::RIST_LOG_DEBUG as c_int, 7);

        assert_eq!(
            rist_connection_status::RIST_CONNECTION_ESTABLISHED as c_int,
            0
        );
    }

    /// Ensure key structs are non-zero-sized (catches obviously wrong layouts).
    #[test]
    fn struct_sizes_nonzero() {
        assert!(core::mem::size_of::<rist_data_block>() > 0);
        assert!(core::mem::size_of::<rist_peer_config>() > 0);
        assert!(core::mem::size_of::<rist_logging_settings>() > 0);
        assert!(core::mem::size_of::<rist_stats>() > 0);
    }

    /// Verify that opaque types have the correct size.
    ///
    /// Note: The !Send and !Sync bounds are enforced via `unsafe impl` in
    /// bindings.rs. If those traits were to be implemented incorrectly,
    /// the code would fail to compile.
    #[test]
    fn opaque_types_zero_sized() {
        assert_eq!(core::mem::size_of::<rist_ctx>(), 0);
        assert_eq!(core::mem::size_of::<rist_peer>(), 0);
    }
}
