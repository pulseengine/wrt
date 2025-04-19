/// Memory access is out of bounds
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAccessOutOfBounds {
    /// Address accessed
    pub address: u64,
    /// Length of access
    pub length: u64,
} 