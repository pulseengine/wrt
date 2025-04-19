//! Version information for wrt-format.

/// Current state serialization format version
pub const STATE_VERSION: u32 = 1;

/// Magic bytes that identify WRT state sections
pub const STATE_MAGIC: &[u8; 4] = b"WRT\0";
