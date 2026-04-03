//! Safe wrappers around librist FFI functions
//!
//! This module provides safer alternatives to raw FFI bindings,
//! performing null checks and returning proper `Result` types.
//! It also provides ownership-based wrapper types with automatic cleanup.
//!
//! # Usage
//!
//! ```rust
//! use librist_sys::safe::{Context, Sender};
//!
//! let sender = Sender::new(librist_sys::rist_profile::RIST_PROFILE_SIMPLE, 0)?;
//! sender.start()?;
//! // Automatically cleaned up when dropped
//! ```

// Re-export error helpers from error module to avoid duplication
use crate::error::{null_check, to_result, Error, Result};
use crate::{
    connection_status_callback_t, receiver_data_callback2_t, rist_ctx, rist_data_block,
    rist_logging_settings, rist_peer, rist_peer_config, rist_profile, rist_stats,
    rist_stats_callback_t,
};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

// ============================================================================
// Owned Context Handle
// ============================================================================

/// A RIST context with ownership semantics.
///
/// This type wraps a raw `rist_ctx` pointer and automatically calls
/// `rist_destroy` when dropped.
///
/// # Example
///
/// ```rust
/// use librist_sys::safe::{Context, Profile};
///
/// {
///     let mut ctx = Context::new_sender(Profile::Simple, 0)?;
///     ctx.start()?;
///     // ... use context ...
/// } // automatically destroyed here
/// ```
pub struct Context {
    ptr: *mut rist_ctx,
}

impl Context {
    /// Create a new RIST sender context.
    pub fn new_sender(
        profile: rist_profile,
        flow_id: u32,
        logging: Option<&mut rist_logging_settings>,
    ) -> Result<Self> {
        let mut ctx: *mut rist_ctx = ptr::null_mut();
        let log_ptr = logging.map_or(ptr::null_mut(), |l| l as *mut _);

        unsafe {
            let ret = crate::rist_sender_create(&mut ctx, profile, flow_id, log_ptr);
            to_result(ret)?;
            null_check(ctx).map(|ptr| Self { ptr })
        }
    }

    /// Create a new RIST receiver context.
    pub fn new_receiver(
        profile: rist_profile,
        logging: Option<&mut rist_logging_settings>,
    ) -> Result<Self> {
        let mut ctx: *mut rist_ctx = ptr::null_mut();
        let log_ptr = logging.map_or(ptr::null_mut(), |l| l as *mut _);

        unsafe {
            let ret = crate::rist_receiver_create(&mut ctx, profile, log_ptr);
            to_result(ret)?;
            null_check(ctx).map(|ptr| Self { ptr })
        }
    }

    /// Start the RIST context.
    pub fn start(&mut self) -> Result<()> {
        unsafe {
            let ret = crate::rist_start(self.ptr);
            to_result(ret).map(|_| ())
        }
    }

    /// Get the raw context pointer (for use with FFI functions that require it).
    #[inline]
    pub fn as_ptr(&self) -> *mut rist_ctx {
        self.ptr
    }

    /// Get a mutable reference to the raw context pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut rist_ctx {
        self.ptr
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            let _ = crate::rist_destroy(self.ptr);
        }
    }
}

// ============================================================================
// Owned Peer Handle
// ============================================================================

/// A RIST peer with ownership semantics.
///
/// This type wraps a raw `rist_peer` pointer and automatically calls
/// `rist_peer_destroy` when dropped. The lifetime `'ctx` ensures the
/// context remains valid for the lifetime of this peer.
pub struct Peer<'ctx> {
    /// Reference to the context (not raw pointer, so borrow checker tracks lifetime)
    _ctx: &'ctx Context,
    ptr: *mut rist_peer,
}

impl<'ctx> Peer<'ctx> {
    /// Create a new peer and attach it to the given context.
    ///
    /// The `ctx` parameter must outlive this Peer instance.
    pub fn create(ctx: &'ctx Context, config: *const rist_peer_config) -> Result<Self> {
        let mut peer: *mut rist_peer = ptr::null_mut();
        unsafe {
            let ret = crate::rist_peer_create(ctx.as_ptr(), &mut peer, config);
            to_result(ret)?;
            null_check(peer).map(|ptr| Self { _ctx: ctx, ptr })
        }
    }

    /// Get the raw peer pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut rist_peer {
        self.ptr
    }

    /// Get a mutable reference to the raw peer pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut rist_peer {
        self.ptr
    }
}

impl Drop for Peer<'_> {
    fn drop(&mut self) {
        // Use the Context's pointer for destruction
        // Note: This is safe because the Context's _ctx borrow ensures
        // the context is still alive when Peer is dropped
        unsafe {
            // We need to access the context through a raw pointer since
            // we're in a drop impl and can't move self._ctx
            // The Context MUST outlive this Peer, which is enforced by the lifetime
            let _ = crate::rist_peer_destroy(self._ctx.as_ptr(), self.ptr);
        }
    }
}

// ============================================================================
// Sender Helpers
// ============================================================================

/// Extension trait for sender-specific operations on Context.
pub trait SenderExt {
    /// Write data to the sender.
    unsafe fn sender_write(&mut self, data_block: *const rist_data_block) -> Result<()>;

    /// Get the flow ID for this sender.
    fn sender_flow_id(&self) -> Result<u32>;

    /// Set the flow ID for this sender.
    fn sender_set_flow_id(&mut self, flow_id: u32) -> Result<()>;

    /// Enable null-packet deletion (NPD).
    fn sender_npd_enable(&mut self) -> Result<()>;
    
    /// Disable null-packet deletion (NPD).
    fn sender_npd_disable(&mut self) -> Result<()>;
}

impl SenderExt for Context {
    unsafe fn sender_write(&mut self, data_block: *const rist_data_block) -> Result<()> {
        let data_block = null_check(data_block as *mut rist_data_block)?;
        to_result(crate::rist_sender_data_write(self.ptr, data_block)).map(|_| ())
    }

    fn sender_flow_id(&self) -> Result<u32> {
        unsafe {
            let mut flow_id: u32 = 0;
            let ret = crate::rist_sender_flow_id_get(self.ptr, &mut flow_id);
            to_result(ret).map(|_| flow_id)
        }
    }

    fn sender_set_flow_id(&mut self, flow_id: u32) -> Result<()> {
        unsafe {
            let ret = crate::rist_sender_flow_id_set(self.ptr, flow_id);
            to_result(ret).map(|_| ())
        }
    }

    fn sender_npd_enable(&mut self) -> Result<()> {
        unsafe {
            let ret = crate::rist_sender_npd_enable(self.ptr);
            to_result(ret).map(|_| ())
        }
    }

    fn sender_npd_disable(&mut self) -> Result<()> {
        unsafe {
            let ret = crate::rist_sender_npd_disable(self.ptr);
            to_result(ret).map(|_| ())
        }
    }
}

// ============================================================================
// Receiver Helpers
// ============================================================================

/// Extension trait for receiver-specific operations on Context.
pub trait ReceiverExt {
    /// Read data from the receiver (blocking).
    unsafe fn receiver_read(&mut self, timeout: c_int) -> Result<*mut rist_data_block>;

    /// Set the data callback for receiver.
    fn receiver_set_data_callback(
        &mut self,
        callback: receiver_data_callback2_t,
        arg: *mut c_void,
    ) -> Result<c_int>;

    /// Free a data block received from receiver.
    fn receiver_free_data_block(block: *mut *mut rist_data_block) -> Result<()>;

    /// Set the NACK type for receiver.
    fn receiver_set_nack_type(&mut self, nack_type: c_int) -> Result<()>;
}

impl ReceiverExt for Context {
    unsafe fn receiver_read(&mut self, timeout: c_int) -> Result<*mut rist_data_block> {
        let mut data_block: *mut rist_data_block = ptr::null_mut();
        let ret = crate::rist_receiver_data_read2(self.ptr, &mut data_block, timeout);
        to_result(ret)?;
        null_check(data_block)
    }

    fn receiver_set_data_callback(
        &mut self,
        callback: receiver_data_callback2_t,
        arg: *mut c_void,
    ) -> Result<()> {
        unsafe {
            let ret = crate::rist_receiver_data_callback_set2(self.ptr, callback, arg);
            to_result(ret).map(|_| ())
        }
    }

    fn receiver_free_data_block(block: *mut *mut rist_data_block) -> Result<()> {
        if block.is_null() {
            return Err(Error::NullPointer);
        }
        unsafe {
            crate::rist_receiver_data_block_free2(block);
        }
        Ok(())
    }

    fn receiver_set_nack_type(&mut self, nack_type: c_int) -> Result<()> {
        unsafe {
            let ret = crate::rist_receiver_nack_type_set(self.ptr, nack_type);
            to_result(ret).map(|_| ())
        }
    }
}

// ============================================================================
// Peer Configuration Helpers
// ============================================================================

/// Peer configuration helpers.
pub mod peer {
    use super::*;

    /// Set default values for a peer configuration.
    pub unsafe fn config_defaults(config: *mut rist_peer_config) -> Result<c_int> {
        let config = null_check(config)?;
        to_result(crate::rist_peer_config_defaults_set(config))
    }

    /// Parse a RIST URL into a peer config.
    pub unsafe fn parse_address(url: &str) -> Result<*mut rist_peer_config> {
        let url_c = std::ffi::CString::new(url).map_err(|_| Error::InvalidArgument)?;
        let mut peer_config: *mut rist_peer_config = ptr::null_mut();
        let ret = crate::rist_parse_address2(url_c.as_ptr(), &mut peer_config);
        to_result(ret)?;
        null_check(peer_config)
    }

    /// Free a peer configuration.
    pub unsafe fn config_free(config: *mut *mut rist_peer_config) -> Result<()> {
        if config.is_null() {
            return Err(Error::NullPointer);
        }
        to_result(crate::rist_peer_config_free2(config)).map(|_| ())
    }
}

// ============================================================================
// Logging Helpers
// ============================================================================

/// Logging configuration helpers.
pub mod logging {
    use super::*;

    /// Configure logging settings.
    pub unsafe fn set(
        log_level: crate::rist_log_level,
        log_cb: Option<
            unsafe extern "C" fn(*mut c_void, crate::rist_log_level, *const c_char) -> c_int,
        >,
        cb_arg: *mut c_void,
    ) -> Result<*mut rist_logging_settings> {
        let mut logging_settings: *mut rist_logging_settings = ptr::null_mut();
        let ret = crate::rist_logging_set(
            &mut logging_settings,
            log_level,
            log_cb,
            cb_arg,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        to_result(ret)?;
        null_check(logging_settings)
    }

    /// Free logging settings.
    pub unsafe fn settings_free(settings: *mut *mut rist_logging_settings) -> Result<()> {
        if settings.is_null() {
            return Err(Error::NullPointer);
        }
        to_result(crate::rist_logging_settings_free2(settings)).map(|_| ())
    }
}

// ============================================================================
// Statistics Helpers
// ============================================================================

/// Statistics helpers.
pub mod stats {
    use super::*;

    /// Set the statistics callback.
    pub unsafe fn callback_set(
        ctx: *mut rist_ctx,
        interval: c_int,
        callback: rist_stats_callback_t,
        arg: *mut c_void,
    ) -> Result<c_int> {
        let ctx = null_check(ctx)?;
        let ret = crate::rist_stats_callback_set(ctx, interval, callback, arg);
        to_result(ret).map(|_| ())
    }

    /// Free statistics container.
    pub unsafe fn free(stats: *const rist_stats) -> Result<c_int> {
        if stats.is_null() {
            return Err(Error::NullPointer);
        }
        let ret = crate::rist_stats_free(stats);
        to_result(ret).map(|_| ())
    }
}

// ============================================================================
// Connection Status Helpers
// ============================================================================

/// Connection status callback helpers.
pub mod connection {
    use super::*;

    /// Set the connection status callback.
    pub unsafe fn callback_set(
        ctx: *mut rist_ctx,
        callback: connection_status_callback_t,
        arg: *mut c_void,
    ) -> Result<c_int> {
        let ctx = null_check(ctx)?;
        let ret = crate::rist_connection_status_callback_set(ctx, callback, arg);
        to_result(ret).map(|_| ())
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the librist library version string.
pub fn version() -> &'static str {
    unsafe {
        let version = crate::librist_version();
        if version.is_null() {
            return "unknown";
        }
        CStr::from_ptr(version).to_str().unwrap_or("unknown")
    }
}

/// Create a new unique flow ID.
pub fn flow_id_create() -> u32 {
    unsafe { crate::rist_flow_id_create() }
}

// Re-export commonly used types for convenience
pub use crate::rist_data_block as DataBlock;
pub use crate::rist_log_level as LogLevel;
pub use crate::rist_profile as Profile;
