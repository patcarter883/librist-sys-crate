//! FFI type definitions for librist
//!
//! This module contains all the raw FFI types and function declarations
//! needed to interface with the librist C library.

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Opaque Types
// ============================================================================

/// Opaque handle to a RIST sender or receiver context.
///
/// Obtained via `rist_sender_create` or `rist_receiver_create` and released
/// with `rist_destroy`. This type has no fields and cannot be instantiated
/// from Rust - it's only valid behind a raw pointer.
///
/// # Thread Safety
///
/// The underlying C library does not document thread-safety guarantees.
/// Users must synchronize access externally or use the `safe` module's
/// `SyncContext` wrapper.
#[repr(C)]
pub struct rist_ctx {
    _private: [u8; 0],
}

// Note: rist_ctx is not Send or Sync because it wraps a raw pointer to C memory.
// We don't know the thread-safety guarantees of the underlying C library,
// so users must synchronize access externally.

/// Opaque handle to a RIST peer.
///
/// Created via `rist_peer_create` and destroyed with `rist_peer_destroy`.
/// This type has no fields and cannot be instantiated from Rust.
#[repr(C)]
pub struct rist_peer {
    _private: [u8; 0],
}

// Note: rist_peer is not Send or Sync - same reasoning as rist_ctx

/// Opaque handle to an out-of-band data block.
#[repr(C)]
pub struct rist_oob_block {
    _private: [u8; 0],
}

// Note: rist_oob_block is not Send or Sync - same reasoning as rist_ctx

// ============================================================================
// Enums
// ============================================================================

/// RIST transport profile.
///
/// Controls the feature set and encapsulation format used on the wire.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum rist_profile {
    /// Simple profile - basic functionality
    RIST_PROFILE_SIMPLE = 0,
    /// Main profile - extended features
    RIST_PROFILE_MAIN = 1,
    /// Advanced profile - full feature set
    RIST_PROFILE_ADVANCED = 2,
}

/// Log severity level used by the RIST logging subsystem.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum rist_log_level {
    /// Logging disabled
    RIST_LOG_DISABLE = -1,
    /// Error conditions
    RIST_LOG_ERROR = 3,
    /// Warning conditions
    RIST_LOG_WARN = 4,
    /// Normal but significant conditions
    RIST_LOG_NOTICE = 5,
    /// Informational
    RIST_LOG_INFO = 6,
    /// Debug-level messages
    RIST_LOG_DEBUG = 7,
}

/// Packet recovery (ARQ) mode.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum rist_recovery_mode {
    /// Recovery not configured
    RIST_RECOVERY_MODE_UNCONFIGURED = -1,
    /// Recovery disabled
    RIST_RECOVERY_MODE_DISABLED = 0,
    /// Unicast recovery
    RIST_RECOVERY_MODE_UNICAST = 1,
    /// Multicast recovery
    RIST_RECOVERY_MODE_MULTICAST = 2,
}

/// Peer connection status reported via the connection-status callback.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum rist_connection_status {
    /// Connection established successfully
    RIST_CONNECTION_ESTABLISHED = 0,
    /// Connection timed out
    RIST_CONNECTION_TIMED_OUT = 1,
    /// Connection is slow
    RIST_CONNECTION_SLOW = 2,
}

// ============================================================================
// Structures
// ============================================================================

/// A data block passed to or received from the RIST library.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_data_block {
    /// Pointer to the payload bytes.
    pub payload: *const c_void,
    /// Length of the payload in bytes.
    pub payload_len: usize,
    /// NTP timestamp associated with this block.
    pub ts_ntp: u64,
    /// Virtual source port.
    pub virt_src_port: u16,
    /// Virtual destination port.
    pub virt_dst_port: u16,
    /// Peer that sent/receives this block.
    pub peer: *mut rist_peer,
    /// Flow identifier.
    pub flow_id: u32,
    /// Sequence number.
    pub seq: u64,
    /// Flags bitmask.
    pub flags: u32,
    /// Application-owned reference pointer.
    pub r#ref: *mut c_void,
}

/// Configuration for a RIST peer connection.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_peer_config {
    pub version: c_int,
    pub address_family: c_int,
    pub initiate_conn: c_int,
    pub address: [c_char; 256],
    pub miface: [c_char; 128],
    pub physical_port: u16,
    pub virt_dst_port: u16,
    pub recovery_mode: c_int,
    pub recovery_maxbitrate: u32,
    pub recovery_maxbitrate_return: u32,
    pub recovery_length_min: u32,
    pub recovery_length_max: u32,
    pub recovery_reorder_buffer: u32,
    pub recovery_rtt_min: u32,
    pub recovery_rtt_max: u32,
    pub weight: u32,
    pub secret: [c_char; 128],
    pub key_size: c_int,
    pub key_rotation: u32,
    pub compression: c_int,
    pub cname: [c_char; 128],
    pub congestion_control_mode: c_int,
    pub min_retries: u32,
    pub max_retries: u32,
    pub session_timeout: u32,
    pub keepalive_interval: u32,
    pub timing_mode: c_int,
    pub srp_username: [c_char; 256],
    pub srp_password: [c_char; 256],
}

/// Settings for the RIST logging subsystem.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_logging_settings {
    pub log_level: rist_log_level,
    pub log_cb: Option<unsafe extern "C" fn(*mut c_void, rist_log_level, *const c_char) -> c_int>,
    pub log_cb_arg: *mut c_void,
    pub log_socket: c_int,
    pub log_stream: *mut c_void,
}

/// Per-peer statistics reported for a sender flow.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_stats_sender_peer {
    pub cname: [c_char; 128],
    pub peer_id: u32,
    pub bandwidth: usize,
    pub retry_bandwidth: usize,
    pub sent: u64,
    pub received: u64,
    pub retransmitted: u64,
    pub quality: f64,
    pub rtt: u32,
}

/// Per-flow statistics reported for a receiver.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_stats_receiver_flow {
    pub cname: [c_char; 128],
    pub flow_id: u32,
    pub bandwidth: usize,
    pub received: u64,
    pub lost: u64,
    pub recovered: u64,
    pub unrecovered: u64,
    pub reordered: u64,
    pub duplicates: u64,
    pub quality: f64,
}

/// Union of the two possible statistics payloads.
///
/// Which variant is active is determined by `rist_stats::stats_type`:
/// - `0` → `sender_peer`
/// - `1` → `receiver_flow`
///
/// # Safety
///
/// Reading from this union is unsafe in Rust. Use `rist_stats` accessor methods
/// or ensure you check `stats_type` before accessing union fields.
#[repr(C)]
pub union rist_stats_union {
    pub sender_peer: rist_stats_sender_peer,
    pub receiver_flow: rist_stats_receiver_flow,
}

/// Top-level statistics container returned by the stats callback.
///
/// The `stats` field is a union - use the accessor methods to safely access
/// the appropriate variant based on `stats_type`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rist_stats {
    pub json_size: u32,
    pub stats_json: *mut c_char,
    pub version: u16,
    /// `0` for sender peer stats, `1` for receiver flow stats.
    pub stats_type: c_int,
    pub stats: rist_stats_union,
}

impl rist_stats {
    /// Returns which stats variant is active.
    ///
    /// Returns `0` for sender peer stats, `1` for receiver flow stats.
    #[inline]
    pub fn stats_type(&self) -> c_int {
        self.stats_type
    }

    /// Access sender peer stats if available.
    ///
    /// Returns `None` if the stats are for a receiver flow (`stats_type != 0`).
    #[inline]
    pub fn sender_peer(&self) -> Option<&rist_stats_sender_peer> {
        if self.stats_type == 0 {
            // SAFETY: stats_type == 0 means sender_peer variant is active
            Some(unsafe { &self.stats.sender_peer })
        } else {
            None
        }
    }

    /// Access receiver flow stats if available.
    ///
    /// Returns `None` if the stats are for a sender peer (`stats_type != 1`).
    #[inline]
    pub fn receiver_flow(&self) -> Option<&rist_stats_receiver_flow> {
        if self.stats_type == 1 {
            // SAFETY: stats_type == 1 means receiver_flow variant is active
            Some(unsafe { &self.stats.receiver_flow })
        } else {
            None
        }
    }

    /// Access the raw union for advanced use cases.
    ///
    /// # Safety
    /// The caller must ensure they know which variant is active by checking
    /// `stats_type()` first. Reading the wrong variant is undefined behavior.
    #[inline]
    pub unsafe fn stats_union(&self) -> &rist_stats_union {
        &self.stats
    }
}

// ============================================================================
// Callback Types
// ============================================================================

/// Callback invoked when data is received (version 2 API).
pub type receiver_data_callback2_t =
    Option<unsafe extern "C" fn(arg: *mut c_void, data_block: *mut rist_data_block) -> c_int>;

/// Callback invoked when a peer's connection status changes.
pub type connection_status_callback_t =
    Option<unsafe extern "C" fn(arg: *mut c_void, peer: *mut rist_peer, status: c_int)>;

/// Callback invoked periodically with statistics.
pub type rist_stats_callback_t =
    Option<unsafe extern "C" fn(arg: *mut c_void, stats: *const rist_stats) -> c_int>;

// ============================================================================
// External Functions
// ============================================================================

extern "C" {
    // -- Core lifecycle -------------------------------------------------------

    /// Start a previously-created RIST context (sender or receiver).
    pub fn rist_start(ctx: *mut rist_ctx) -> c_int;

    /// Destroy a RIST context and free all associated resources.
    pub fn rist_destroy(ctx: *mut rist_ctx) -> c_int;

    // -- Sender ---------------------------------------------------------------

    /// Create a new RIST sender context.
    pub fn rist_sender_create(
        ctx: *mut *mut rist_ctx,
        profile: rist_profile,
        flow_id: u32,
        logging_settings: *mut rist_logging_settings,
    ) -> c_int;

    /// Write a data block to the sender for transmission.
    pub fn rist_sender_data_write(ctx: *mut rist_ctx, data_block: *const rist_data_block) -> c_int;

    /// Get the current flow ID for a sender context.
    pub fn rist_sender_flow_id_get(ctx: *mut rist_ctx, flow_id: *mut u32) -> c_int;

    /// Set the flow ID for a sender context.
    pub fn rist_sender_flow_id_set(ctx: *mut rist_ctx, flow_id: u32) -> c_int;

    /// Enable null-packet deletion on the sender.
    pub fn rist_sender_npd_enable(ctx: *mut rist_ctx) -> c_int;

    /// Disable null-packet deletion on the sender.
    pub fn rist_sender_npd_disable(ctx: *mut rist_ctx) -> c_int;

    // -- Receiver -------------------------------------------------------------

    /// Create a new RIST receiver context.
    pub fn rist_receiver_create(
        ctx: *mut *mut rist_ctx,
        profile: rist_profile,
        logging_settings: *mut rist_logging_settings,
    ) -> c_int;

    /// Blocking read of the next data block from the receiver.
    pub fn rist_receiver_data_read2(
        ctx: *mut rist_ctx,
        data_block: *mut *mut rist_data_block,
        timeout: c_int,
    ) -> c_int;

    /// Register a callback for incoming data on the receiver.
    pub fn rist_receiver_data_callback_set2(
        ctx: *mut rist_ctx,
        callback: receiver_data_callback2_t,
        arg: *mut c_void,
    ) -> c_int;

    /// Free a data block previously obtained via `rist_receiver_data_read2`.
    pub fn rist_receiver_data_block_free2(block: *mut *mut rist_data_block);

    /// Set the NACK type for the receiver.
    pub fn rist_receiver_nack_type_set(ctx: *mut rist_ctx, nack_type: c_int) -> c_int;

    // -- Peer management ------------------------------------------------------

    /// Fill a `rist_peer_config` with sensible defaults.
    pub fn rist_peer_config_defaults_set(peer_config: *mut rist_peer_config) -> c_int;

    /// Parse a RIST URL into a newly-allocated peer config.
    pub fn rist_parse_address2(
        url: *const c_char,
        peer_config: *mut *mut rist_peer_config,
    ) -> c_int;

    /// Free a peer config allocated by `rist_parse_address2`.
    pub fn rist_peer_config_free2(peer_config: *mut *mut rist_peer_config) -> c_int;

    /// Create a peer and attach it to the given context.
    pub fn rist_peer_create(
        ctx: *mut rist_ctx,
        peer: *mut *mut rist_peer,
        config: *const rist_peer_config,
    ) -> c_int;

    /// Destroy a peer attached to the given context.
    pub fn rist_peer_destroy(ctx: *mut rist_ctx, peer: *mut rist_peer) -> c_int;

    /// Register a callback for peer connection-status changes.
    pub fn rist_connection_status_callback_set(
        ctx: *mut rist_ctx,
        callback: connection_status_callback_t,
        arg: *mut c_void,
    ) -> c_int;

    // -- Statistics -----------------------------------------------------------

    /// Register a periodic statistics callback.
    pub fn rist_stats_callback_set(
        ctx: *mut rist_ctx,
        statsinterval: c_int,
        stats_cb: rist_stats_callback_t,
        arg: *mut c_void,
    ) -> c_int;

    /// Free a statistics container returned by the stats callback.
    pub fn rist_stats_free(stats_container: *const rist_stats) -> c_int;

    // -- Logging --------------------------------------------------------------

    /// Allocate and configure a new logging-settings handle.
    pub fn rist_logging_set(
        logging_settings: *mut *mut rist_logging_settings,
        log_level: rist_log_level,
        log_cb: Option<unsafe extern "C" fn(*mut c_void, rist_log_level, *const c_char) -> c_int>,
        cb_arg: *mut c_void,
        address: *mut c_char,
        logfp: *mut c_void,
    ) -> c_int;

    /// Free a logging-settings handle allocated by `rist_logging_set`.
    pub fn rist_logging_settings_free2(logging_settings: *mut *mut rist_logging_settings) -> c_int;

    // NOTE: rist_log() is a C variadic function. Declaring variadic externs
    // requires nightly. Consumers who need rist_log() should format the
    // message in Rust and call the logging callback directly.

    // -- Utility --------------------------------------------------------------

    /// Return the librist version string (static lifetime in C).
    pub fn librist_version() -> *const c_char;

    /// Generate a random flow ID suitable for use with `rist_sender_create`.
    pub fn rist_flow_id_create() -> u32;
}

// ============================================================================
// Constants
// ============================================================================

/// Current peer-config struct version expected by the library.
pub const RIST_PEER_CONFIG_VERSION: c_int = 4;

/// Default virtual destination port.
pub const RIST_DEFAULT_VIRT_DST_PORT: u16 = 1968;

/// Default recovery (ARQ) mode - unicast.
pub const RIST_DEFAULT_RECOVERY_MODE: c_int = 1;

/// Default maximum recovery bitrate (100 kbps).
pub const RIST_DEFAULT_RECOVERY_MAXBITRATE: u32 = 100_000;

/// Default maximum recovery bitrate for return path (0 = unlimited).
pub const RIST_DEFAULT_RECOVERY_MAXBITRATE_RETURN: u32 = 0;

/// Default minimum recovery packet distance.
pub const RIST_DEFAULT_RECOVERY_LENGTH_MIN: u32 = 1000;

/// Default maximum recovery packet distance.
pub const RIST_DEFAULT_RECOVERY_LENGTH_MAX: u32 = 1000;

/// Default recovery reordering buffer size.
pub const RIST_DEFAULT_RECOVERY_REORDER_BUFFER: u32 = 0;

/// Default minimum RTT estimate (ms).
pub const RIST_DEFAULT_RECOVERY_RTT_MIN: u32 = 50;

/// Default maximum RTT estimate (ms).
pub const RIST_DEFAULT_RECOVERY_RTT_MAX: u32 = 100;

/// Default congestion control mode.
pub const RIST_DEFAULT_CONGESTION_CONTROL_MODE: c_int = 1;

/// Default minimum retry attempts.
pub const RIST_DEFAULT_MIN_RETRIES: u32 = 6;

/// Default maximum retry attempts.
pub const RIST_DEFAULT_MAX_RETRIES: u32 = 20;

/// Default session timeout in milliseconds.
pub const RIST_DEFAULT_SESSION_TIMEOUT: u32 = 5000;

// ============================================================================
// Additional Constants
// ============================================================================

/// NACK type: Normal (packet-level)
pub const RIST_NACK_TYPE_NORMAL: c_int = 0;

/// NACK type: Range-based (interval-based)
pub const RIST_NACK_TYPE_RANGE: c_int = 1;

/// NACK type: None (disabled)
pub const RIST_NACK_TYPE_NONE: c_int = 2;

/// Timing mode: Source-based (use sender timestamps)
pub const RIST_TIMING_MODE_SOURCE: c_int = 0;

/// Timing mode: Local (use receiver timestamps)
pub const RIST_TIMING_MODE_LOCAL: c_int = 1;

/// Timing mode: Independent (no timing adjustment)
pub const RIST_TIMING_MODE_INDEPENDENT: c_int = 2;

/// Address family: Auto-detect
pub const RIST_ADDRESS_FAMILY_AUTO: c_int = 0;

/// Address family: IPv4
pub const RIST_ADDRESS_FAMILY_IPV4: c_int = 1;

/// Address family: IPv6
pub const RIST_ADDRESS_FAMILY_IPV6: c_int = 2;

/// Compression: None
pub const RIST_COMPRESSION_NONE: c_int = 0;

/// Compression: Zstd
pub const RIST_COMPRESSION_ZSTD: c_int = 1;

/// Compression: LZ4
pub const RIST_COMPRESSION_LZ4: c_int = 2;
