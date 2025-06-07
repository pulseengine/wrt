// Enhanced Bounded Logging Infrastructure for Agent C
// This is Agent C's bounded logging implementation according to the parallel development plan

extern crate alloc;
use alloc::{string::String, vec::Vec};
#[cfg(not(any(feature = "std", feature = "alloc")))]
use core::{fmt, mem};
use wrt_error::{Error, Result};
use crate::level::LogLevel;

/// Bounded logging limits configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedLoggingLimits {
    pub max_log_buffer_size: usize,
    pub max_log_message_size: usize,
    pub max_concurrent_loggers: usize,
    pub max_log_entries: usize,
    pub retention_time_ms: u64,
    pub flush_threshold: usize,
}

impl Default for BoundedLoggingLimits {
    fn default() -> Self {
        Self {
            max_log_buffer_size: 64 * 1024,  // 64KB
            max_log_message_size: 1024,      // 1KB per message
            max_concurrent_loggers: 16,      // 16 concurrent loggers
            max_log_entries: 1000,           // 1000 log entries
            retention_time_ms: 300_000,      // 5 minutes
            flush_threshold: 100,            // Flush after 100 entries
        }
    }
}

impl BoundedLoggingLimits {
    /// Create limits for embedded platforms
    pub fn embedded() -> Self {
        Self {
            max_log_buffer_size: 8 * 1024,   // 8KB
            max_log_message_size: 256,       // 256B per message
            max_concurrent_loggers: 4,       // 4 concurrent loggers
            max_log_entries: 100,            // 100 log entries
            retention_time_ms: 60_000,       // 1 minute
            flush_threshold: 20,             // Flush after 20 entries
        }
    }
    
    /// Create limits for QNX platforms
    pub fn qnx() -> Self {
        Self {
            max_log_buffer_size: 32 * 1024,  // 32KB
            max_log_message_size: 512,       // 512B per message
            max_concurrent_loggers: 8,       // 8 concurrent loggers
            max_log_entries: 500,            // 500 log entries
            retention_time_ms: 180_000,      // 3 minutes
            flush_threshold: 50,             // Flush after 50 entries
        }
    }
    
    /// Validate limits are reasonable
    pub fn validate(&self) -> Result<()> {
        if self.max_log_buffer_size == 0 {
            return Err(Error::invalid_input("max_log_buffer_size cannot be zero"));
        }
        if self.max_log_message_size == 0 {
            return Err(Error::invalid_input("max_log_message_size cannot be zero"));
        }
        if self.max_log_message_size > self.max_log_buffer_size {
            return Err(Error::invalid_input("max_log_message_size cannot exceed max_log_buffer_size"));
        }
        if self.max_concurrent_loggers == 0 {
            return Err(Error::invalid_input("max_concurrent_loggers cannot be zero"));
        }
        Ok(())
    }
}

/// Logger identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoggerId(pub u32);

/// Component instance identifier for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentLoggingId(pub u32);

/// Bounded log entry
#[derive(Debug, Clone)]
pub struct BoundedLogEntry {
    pub id: u64,
    pub timestamp: u64,
    pub level: LogLevel,
    pub logger_id: LoggerId,
    pub component_id: ComponentLoggingId,
    pub message: String,
    pub metadata: LogMetadata,
}

/// Log metadata for tracking and filtering
#[derive(Debug, Clone)]
pub struct LogMetadata {
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub thread_id: Option<u32>,
    pub safety_level: u8,
}

impl Default for LogMetadata {
    fn default() -> Self {
        Self {
            module: None,
            file: None,
            line: None,
            thread_id: None,
            safety_level: 0, // QM
        }
    }
}

/// Bounded log buffer for storing log entries
pub struct BoundedLogBuffer {
    entries: Vec<BoundedLogEntry>,
    max_entries: usize,
    buffer_size: usize,
    max_buffer_size: usize,
    next_entry_id: u64,
}

impl BoundedLogBuffer {
    pub fn new(max_entries: usize, max_buffer_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
            buffer_size: 0,
            max_buffer_size,
            next_entry_id: 1,
        }
    }
    
    pub fn add_entry(&mut self, mut entry: BoundedLogEntry) -> Result<()> {
        let entry_size = entry.message.len() + 
            entry.metadata.module.as_ref().map_or(0, |s| s.len()) +
            entry.metadata.file.as_ref().map_or(0, |s| s.len()) +
            64; // Base overhead
        
        // Check if adding this entry would exceed buffer size
        if self.buffer_size + entry_size > self.max_buffer_size {
            self.make_space(entry_size)?;
        }
        
        // Check if we're at max entries
        if self.entries.len() >= self.max_entries {
            self.remove_oldest_entry();
        }
        
        entry.id = self.next_entry_id;
        self.next_entry_id = self.next_entry_id.wrapping_add(1);
        
        self.buffer_size += entry_size;
        self.entries.push(entry);
        
        Ok(())
    }
    
    fn make_space(&mut self, required_size: usize) -> Result<()> {
        while self.buffer_size + required_size > self.max_buffer_size && !self.entries.is_empty() {
            self.remove_oldest_entry();
        }
        
        if self.buffer_size + required_size > self.max_buffer_size {
            return Err(Error::OUT_OF_MEMORY);
        }
        
        Ok(())
    }
    
    fn remove_oldest_entry(&mut self) {
        if let Some(entry) = self.entries.first() {
            let entry_size = entry.message.len() + 
                entry.metadata.module.as_ref().map_or(0, |s| s.len()) +
                entry.metadata.file.as_ref().map_or(0, |s| s.len()) +
                64;
            self.buffer_size = self.buffer_size.saturating_sub(entry_size);
        }
        
        if !self.entries.is_empty() {
            self.entries.remove(0);
        }
    }
    
    pub fn get_entries(&self) -> &[BoundedLogEntry] {
        &self.entries
    }
    
    pub fn get_entries_by_level(&self, level: LogLevel) -> Vec<&BoundedLogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
    
    pub fn get_entries_by_component(&self, component_id: ComponentLoggingId) -> Vec<&BoundedLogEntry> {
        self.entries.iter()
            .filter(|entry| entry.component_id == component_id)
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.buffer_size = 0;
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

/// Bounded logger instance
pub struct BoundedLogger {
    pub id: LoggerId,
    pub component_id: ComponentLoggingId,
    pub name: String,
    pub min_level: LogLevel,
    pub enabled: bool,
    pub message_count: u64,
}

impl BoundedLogger {
    pub fn new(
        id: LoggerId,
        component_id: ComponentLoggingId,
        name: String,
        min_level: LogLevel,
    ) -> Self {
        Self {
            id,
            component_id,
            name,
            min_level,
            enabled: true,
            message_count: 0,
        }
    }
    
    pub fn should_log(&self, level: LogLevel) -> bool {
        self.enabled && level >= self.min_level
    }
    
    pub fn increment_message_count(&mut self) {
        self.message_count = self.message_count.wrapping_add(1);
    }
}

/// Bounded logging manager
pub struct BoundedLoggingManager {
    limits: BoundedLoggingLimits,
    buffer: BoundedLogBuffer,
    loggers: Vec<BoundedLogger>,
    next_logger_id: u32,
    total_messages: u64,
    dropped_messages: u64,
    flush_pending: bool,
}

impl BoundedLoggingManager {
    /// Create a new bounded logging manager
    pub fn new(limits: BoundedLoggingLimits) -> Result<Self> {
        limits.validate()?;
        
        let buffer = BoundedLogBuffer::new(limits.max_log_entries, limits.max_log_buffer_size);
        
        Ok(Self {
            limits,
            buffer,
            loggers: Vec::new(),
            next_logger_id: 1,
            total_messages: 0,
            dropped_messages: 0,
            flush_pending: false,
        })
    }
    
    /// Register a new logger
    pub fn register_logger(
        &mut self,
        component_id: ComponentLoggingId,
        name: String,
        min_level: LogLevel,
    ) -> Result<LoggerId> {
        // Check logger limit
        if self.loggers.len() >= self.limits.max_concurrent_loggers {
            return Err(Error::TOO_MANY_COMPONENTS);
        }
        
        let logger_id = LoggerId(self.next_logger_id);
        self.next_logger_id = self.next_logger_id.wrapping_add(1);
        
        let logger = BoundedLogger::new(logger_id, component_id, name, min_level);
        self.loggers.push(logger);
        
        Ok(logger_id)
    }
    
    /// Log a message with bounds checking
    pub fn log_message(
        &mut self,
        logger_id: LoggerId,
        level: LogLevel,
        message: String,
        metadata: LogMetadata,
    ) -> Result<()> {
        // Check message size limit
        if message.len() > self.limits.max_log_message_size {
            self.dropped_messages += 1;
            return Err(Error::invalid_input("Log message too large"));
        }
        
        // Find the logger and get its component_id
        let (component_id, should_log) = {
            let logger = self.loggers.iter()
                .find(|logger| logger.id == logger_id)
                .ok_or(Error::COMPONENT_NOT_FOUND)?;
            
            (logger.component_id, logger.should_log(level))
        };
        
        // Check if logger should log this level
        if !should_log {
            return Ok(()); // Silently ignore
        }
        
        // Create log entry
        let entry = BoundedLogEntry {
            id: 0, // Will be set by buffer
            timestamp: self.get_timestamp(),
            level,
            logger_id,
            component_id,
            message,
            metadata,
        };
        
        // Add to buffer
        match self.buffer.add_entry(entry) {
            Ok(()) => {
                // Find and update the logger's message count
                if let Some(logger) = self.loggers.iter_mut().find(|l| l.id == logger_id) {
                    logger.increment_message_count();
                }
                self.total_messages += 1;
                
                // Check if we should flush
                if self.buffer.len() >= self.limits.flush_threshold {
                    self.flush_pending = true;
                }
            }
            Err(_) => {
                self.dropped_messages += 1;
                return Err(Error::OUT_OF_MEMORY);
            }
        }
        
        Ok(())
    }
    
    /// Convenience method for logging with minimal metadata
    pub fn log(
        &mut self,
        logger_id: LoggerId,
        level: LogLevel,
        message: String,
    ) -> Result<()> {
        self.log_message(logger_id, level, message, LogMetadata::default())
    }
    
    /// Get logger by ID
    pub fn get_logger(&self, logger_id: LoggerId) -> Option<&BoundedLogger> {
        self.loggers.iter().find(|logger| logger.id == logger_id)
    }
    
    /// Get mutable logger by ID
    pub fn get_logger_mut(&mut self, logger_id: LoggerId) -> Option<&mut BoundedLogger> {
        self.loggers.iter_mut().find(|logger| logger.id == logger_id)
    }
    
    /// Enable/disable a logger
    pub fn set_logger_enabled(&mut self, logger_id: LoggerId, enabled: bool) -> Result<()> {
        let logger = self.get_logger_mut(logger_id)
            .ok_or(Error::COMPONENT_NOT_FOUND)?;
        logger.enabled = enabled;
        Ok(())
    }
    
    /// Set minimum log level for a logger
    pub fn set_logger_level(&mut self, logger_id: LoggerId, min_level: LogLevel) -> Result<()> {
        let logger = self.get_logger_mut(logger_id)
            .ok_or(Error::COMPONENT_NOT_FOUND)?;
        logger.min_level = min_level;
        Ok(())
    }
    
    /// Get log entries
    pub fn get_log_entries(&self) -> &[BoundedLogEntry] {
        self.buffer.get_entries()
    }
    
    /// Get log entries by level
    pub fn get_entries_by_level(&self, level: LogLevel) -> Vec<&BoundedLogEntry> {
        self.buffer.get_entries_by_level(level)
    }
    
    /// Get log entries by component
    pub fn get_entries_by_component(&self, component_id: ComponentLoggingId) -> Vec<&BoundedLogEntry> {
        self.buffer.get_entries_by_component(component_id)
    }
    
    /// Clear all log entries
    pub fn clear_logs(&mut self) {
        self.buffer.clear();
        self.flush_pending = false;
    }
    
    /// Remove all loggers for a component
    pub fn remove_component_loggers(&mut self, component_id: ComponentLoggingId) -> usize {
        let initial_count = self.loggers.len();
        self.loggers.retain(|logger| logger.component_id != component_id);
        initial_count - self.loggers.len()
    }
    
    /// Check if flush is pending
    pub fn is_flush_pending(&self) -> bool {
        self.flush_pending
    }
    
    /// Mark flush as completed
    pub fn mark_flushed(&mut self) {
        self.flush_pending = false;
    }
    
    /// Get logging statistics
    pub fn get_statistics(&self) -> BoundedLoggingStatistics {
        let memory_used = self.buffer.buffer_size();
        let memory_utilization = if self.limits.max_log_buffer_size > 0 {
            (memory_used as f64 / self.limits.max_log_buffer_size as f64) * 100.0
        } else {
            0.0
        };
        
        BoundedLoggingStatistics {
            registered_loggers: self.loggers.len(),
            active_loggers: self.loggers.iter().filter(|l| l.enabled).count(),
            total_log_entries: self.buffer.len(),
            memory_used,
            memory_utilization,
            total_messages: self.total_messages,
            dropped_messages: self.dropped_messages,
            flush_pending: self.flush_pending,
        }
    }
    
    /// Validate all logging state
    pub fn validate(&self) -> Result<()> {
        if self.loggers.len() > self.limits.max_concurrent_loggers {
            return Err(Error::TOO_MANY_COMPONENTS);
        }
        
        if self.buffer.buffer_size() > self.limits.max_log_buffer_size {
            return Err(Error::OUT_OF_MEMORY);
        }
        
        if self.buffer.len() > self.limits.max_log_entries {
            return Err(Error::OUT_OF_MEMORY);
        }
        
        Ok(())
    }
    
    /// Get timestamp (stub implementation)
    fn get_timestamp(&self) -> u64 {
        // In a real implementation, this would use platform-specific timing
        0
    }
}

/// Logging statistics
#[derive(Debug, Clone)]
pub struct BoundedLoggingStatistics {
    pub registered_loggers: usize,
    pub active_loggers: usize,
    pub total_log_entries: usize,
    pub memory_used: usize,
    pub memory_utilization: f64, // Percentage
    pub total_messages: u64,
    pub dropped_messages: u64,
    pub flush_pending: bool,
}

/// Convenience macros for logging (only available with alloc)

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($manager:expr, $logger_id:expr, $($arg:tt)*) => {
        $manager.log($logger_id, $crate::LogLevel::Debug, alloc::format!($($arg)*))
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($manager:expr, $logger_id:expr, $($arg:tt)*) => {
        $manager.log($logger_id, $crate::LogLevel::Info, alloc::format!($($arg)*))
    };
}

/// Log a warning message
#[macro_export]
macro_rules! log_warning {
    ($manager:expr, $logger_id:expr, $($arg:tt)*) => {
        $manager.log($logger_id, $crate::LogLevel::Warning, alloc::format!($($arg)*))
    };
}

/// Log an error message
#[macro_export]
macro_rules! log_error {
    ($manager:expr, $logger_id:expr, $($arg:tt)*) => {
        $manager.log($logger_id, $crate::LogLevel::Error, alloc::format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bounded_logging_manager_creation() {
        let limits = BoundedLoggingLimits::default();
        let manager = BoundedLoggingManager::new(limits);
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let stats = manager.get_statistics();
        assert_eq!(stats.registered_loggers, 0);
        assert_eq!(stats.total_log_entries, 0);
    }
    
    #[test]
    fn test_logger_registration() {
        let limits = BoundedLoggingLimits::default();
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger_id = manager.register_logger(
            ComponentLoggingId(1),
            "test-logger".to_string(),
            LogLevel::Info,
        ).unwrap();
        
        assert_eq!(logger_id.0, 1);
        
        let stats = manager.get_statistics();
        assert_eq!(stats.registered_loggers, 1);
        assert_eq!(stats.active_loggers, 1);
    }
    
    #[test]
    fn test_log_message() {
        let limits = BoundedLoggingLimits::default();
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger_id = manager.register_logger(
            ComponentLoggingId(1),
            "test-logger".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        let result = manager.log(logger_id, LogLevel::Info, "Test message".to_string());
        assert!(result.is_ok());
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_log_entries, 1);
        assert_eq!(stats.total_messages, 1);
    }
    
    #[test]
    fn test_log_level_filtering() {
        let limits = BoundedLoggingLimits::default();
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger_id = manager.register_logger(
            ComponentLoggingId(1),
            "test-logger".to_string(),
            LogLevel::Warning, // Only log Warning and Error
        ).unwrap();
        
        // This should be ignored (Debug < Warning)
        let result = manager.log(logger_id, LogLevel::Debug, "Debug message".to_string());
        assert!(result.is_ok());
        
        // This should be logged (Warning >= Warning)
        let result = manager.log(logger_id, LogLevel::Warning, "Warning message".to_string());
        assert!(result.is_ok());
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_log_entries, 1); // Only the warning message
        assert_eq!(stats.total_messages, 1);
    }
    
    #[test]
    fn test_message_size_limits() {
        let limits = BoundedLoggingLimits {
            max_log_message_size: 10,
            ..BoundedLoggingLimits::default()
        };
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger_id = manager.register_logger(
            ComponentLoggingId(1),
            "test-logger".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        // This message is too long (20 chars > 10 limit)
        let result = manager.log(logger_id, LogLevel::Info, "This message is too long".to_string());
        assert!(result.is_err());
        
        let stats = manager.get_statistics();
        assert_eq!(stats.dropped_messages, 1);
    }
    
    #[test]
    fn test_buffer_size_limits() {
        let limits = BoundedLoggingLimits {
            max_log_entries: 2,
            ..BoundedLoggingLimits::default()
        };
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger_id = manager.register_logger(
            ComponentLoggingId(1),
            "test-logger".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        // Add three messages (should only keep the last two)
        manager.log(logger_id, LogLevel::Info, "Message 1".to_string()).unwrap();
        manager.log(logger_id, LogLevel::Info, "Message 2".to_string()).unwrap();
        manager.log(logger_id, LogLevel::Info, "Message 3".to_string()).unwrap();
        
        let entries = manager.get_log_entries();
        assert_eq!(entries.len(), 2);
        
        // Should have the last two messages
        assert_eq!(entries[0].message, "Message 2");
        assert_eq!(entries[1].message, "Message 3");
    }
    
    #[test]
    fn test_logger_limits() {
        let limits = BoundedLoggingLimits {
            max_concurrent_loggers: 1,
            ..BoundedLoggingLimits::default()
        };
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        // First logger should succeed
        let result1 = manager.register_logger(
            ComponentLoggingId(1),
            "logger1".to_string(),
            LogLevel::Debug,
        );
        assert!(result1.is_ok());
        
        // Second logger should fail
        let result2 = manager.register_logger(
            ComponentLoggingId(2),
            "logger2".to_string(),
            LogLevel::Debug,
        );
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_component_logger_removal() {
        let limits = BoundedLoggingLimits::default();
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger1_id = manager.register_logger(
            ComponentLoggingId(1),
            "logger1".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        let logger2_id = manager.register_logger(
            ComponentLoggingId(1),
            "logger2".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        let logger3_id = manager.register_logger(
            ComponentLoggingId(2),
            "logger3".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        let removed = manager.remove_component_loggers(ComponentLoggingId(1));
        assert_eq!(removed, 2);
        
        let stats = manager.get_statistics();
        assert_eq!(stats.registered_loggers, 1);
        
        // Logger3 should still exist
        assert!(manager.get_logger(logger3_id).is_some());
        // Logger1 and Logger2 should be gone
        assert!(manager.get_logger(logger1_id).is_none());
        assert!(manager.get_logger(logger2_id).is_none());
    }
    
    #[test]
    fn test_log_filtering_by_component() {
        let limits = BoundedLoggingLimits::default();
        let mut manager = BoundedLoggingManager::new(limits).unwrap();
        
        let logger1_id = manager.register_logger(
            ComponentLoggingId(1),
            "logger1".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        let logger2_id = manager.register_logger(
            ComponentLoggingId(2),
            "logger2".to_string(),
            LogLevel::Debug,
        ).unwrap();
        
        manager.log(logger1_id, LogLevel::Info, "Message from component 1".to_string()).unwrap();
        manager.log(logger2_id, LogLevel::Info, "Message from component 2".to_string()).unwrap();
        manager.log(logger1_id, LogLevel::Error, "Error from component 1".to_string()).unwrap();
        
        let component1_entries = manager.get_entries_by_component(ComponentLoggingId(1));
        let component2_entries = manager.get_entries_by_component(ComponentLoggingId(2));
        
        assert_eq!(component1_entries.len(), 2);
        assert_eq!(component2_entries.len(), 1);
    }
}