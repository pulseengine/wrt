//! VxWorks-specific synchronization primitives
//!
//! This module provides external implementations of synchronization primitives
//! for VxWorks that demonstrate how to extend wrt-platform with custom sync support.

use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;
use wrt_platform::FutexLike;
use wrt_error::{Error, ErrorKind};

/// VxWorks RTP (Real-Time Process) futex implementation
///
/// Uses POSIX semaphores and mutexes available in VxWorks RTP user-space.
pub struct VxWorksRtpFutex {
    atomic_value: AtomicU32,
    priority_inheritance: bool,
    posix_sem: Option<PosixSemaphore>,
}

/// VxWorks LKM (Loadable Kernel Module) futex implementation
///
/// Uses VxWorks kernel-space semaphores and synchronization primitives.
pub struct VxWorksLkmFutex {
    atomic_value: AtomicU32,
    priority_inheritance: bool,
    vxworks_sem: Option<VxWorksSemaphore>,
}

// Platform-specific semaphore representations
#[repr(C)]
struct PosixSemaphore {
    _data: [u8; 32], // Platform-specific semaphore data
}

#[repr(C)]
struct VxWorksSemaphore {
    sem_id: usize, // VxWorks SEM_ID
}

#[repr(C)]
struct TimeSpec {
    tv_sec: i64,
    tv_nsec: i64,
}

impl VxWorksRtpFutex {
    /// Create a new RTP futex builder
    pub fn new() -> VxWorksRtpFutexBuilder {
        VxWorksRtpFutexBuilder::new()
    }

    /// Initialize POSIX semaphore for RTP context
    fn init_posix_semaphore(&mut self, initial_value: u32) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn sem_init(sem: *mut PosixSemaphore, pshared: i32, value: u32) -> i32;
            }

            let mut posix_sem = PosixSemaphore { _data: [0; 32] };
            let result = unsafe { sem_init(&mut posix_sem, 0, initial_value) };
            
            if result != 0 {
                return Err(Error::new(
                    ErrorKind::Platform,
                    "Failed to initialize POSIX semaphore in RTP"
                ));
            }

            self.posix_sem = Some(posix_sem);
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            let posix_sem = PosixSemaphore { _data: [0; 32] };
            self.posix_sem = Some(posix_sem);
        }

        Ok(())
    }

    /// Convert duration to timespec
    fn duration_to_timespec(duration: Duration) -> TimeSpec {
        TimeSpec {
            tv_sec: duration.as_secs() as i64,
            tv_nsec: duration.subsec_nanos() as i64,
        }
    }
}

impl FutexLike for VxWorksRtpFutex {
    fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<(), Error> {
        // Check if the atomic value matches expected
        let current = self.atomic_value.load(Ordering::Acquire);
        if current != expected {
            return Ok(()); // Value changed, no need to wait
        }

        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref posix_sem) = self.posix_sem {
                extern "C" {
                    fn sem_wait(sem: *const PosixSemaphore) -> i32;
                    fn sem_timedwait(sem: *const PosixSemaphore, timeout: *const TimeSpec) -> i32;
                }

                let result = match timeout {
                    Some(duration) => {
                        let timespec = Self::duration_to_timespec(duration);
                        unsafe { sem_timedwait(posix_sem, &timespec) }
                    }
                    None => {
                        unsafe { sem_wait(posix_sem) }
                    }
                };

                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "VxWorks RTP semaphore wait failed"
                    ));
                }
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation - just simulate a brief wait
            if let Some(duration) = timeout {
                if duration.as_millis() > 0 {
                    // In real implementation, would actually wait
                }
            }
        }

        Ok(())
    }

    fn wake_one(&self) -> Result<u32, Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref posix_sem) = self.posix_sem {
                extern "C" {
                    fn sem_post(sem: *const PosixSemaphore) -> i32;
                }

                let result = unsafe { sem_post(posix_sem) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "VxWorks RTP semaphore post failed"
                    ));
                }
                return Ok(1); // Woke one task
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation
            return Ok(1);
        }

        Ok(0)
    }

    fn wake_all(&self) -> Result<u32, Error> {
        // POSIX semaphores don't have a direct "wake all" operation
        // We implement it by posting multiple times
        let mut woken = 0;
        
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref posix_sem) = self.posix_sem {
                extern "C" {
                    fn sem_post(sem: *const PosixSemaphore) -> i32;
                }

                // Post up to 32 times to wake potential waiters
                for _ in 0..32 {
                    let result = unsafe { sem_post(posix_sem) };
                    if result == 0 {
                        woken += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation
            woken = 4; // Simulate waking some tasks
        }

        Ok(woken)
    }

    fn load(&self, ordering: Ordering) -> u32 {
        self.atomic_value.load(ordering)
    }

    fn store(&self, value: u32, ordering: Ordering) {
        self.atomic_value.store(value, ordering);
    }

    fn compare_exchange_weak(
        &self,
        current: u32,
        new: u32,
        success: Ordering,
        failure: Ordering,
    ) -> Result<u32, u32> {
        self.atomic_value.compare_exchange_weak(current, new, success, failure)
    }
}

impl Drop for VxWorksRtpFutex {
    fn drop(&mut self) {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref mut posix_sem) = self.posix_sem {
                extern "C" {
                    fn sem_destroy(sem: *mut PosixSemaphore) -> i32;
                }
                
                unsafe {
                    sem_destroy(posix_sem);
                }
            }
        }
    }
}

impl VxWorksLkmFutex {
    /// Create a new LKM futex builder
    pub fn new() -> VxWorksLkmFutexBuilder {
        VxWorksLkmFutexBuilder::new()
    }

    /// Initialize VxWorks semaphore for LKM context
    fn init_vxworks_semaphore(&mut self) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn semBCreate(options: i32, initial_state: i32) -> usize;
            }

            // VxWorks semaphore options
            const SEM_Q_PRIORITY: i32 = 0x01;
            const SEM_DELETE_SAFE: i32 = 0x04;
            const SEM_INVERSION_SAFE: i32 = 0x08;

            let mut options = SEM_Q_PRIORITY | SEM_DELETE_SAFE;
            if self.priority_inheritance {
                options |= SEM_INVERSION_SAFE;
            }

            let sem_id = unsafe { semBCreate(options, 0) }; // Start empty
            if sem_id == 0 {
                return Err(Error::new(
                    ErrorKind::Platform,
                    "Failed to create VxWorks semaphore in LKM"
                ));
            }

            self.vxworks_sem = Some(VxWorksSemaphore { sem_id });
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            let vxworks_sem = VxWorksSemaphore { sem_id: 1 }; // Mock semaphore ID
            self.vxworks_sem = Some(vxworks_sem);
        }

        Ok(())
    }

    /// Convert duration to VxWorks ticks
    fn duration_to_ticks(duration: Duration) -> i32 {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn sysClkRateGet() -> i32;
            }
            
            let ticks_per_sec = unsafe { sysClkRateGet() } as u64;
            let total_ms = duration.as_millis() as u64;
            
            if total_ms == 0 {
                return 0; // NO_WAIT
            }
            
            let ticks = (total_ms * ticks_per_sec) / 1000;
            if ticks > i32::MAX as u64 {
                -1 // WAIT_FOREVER
            } else {
                ticks as i32
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            duration.as_millis() as i32
        }
    }
}

impl FutexLike for VxWorksLkmFutex {
    fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<(), Error> {
        // Check if the atomic value matches expected
        let current = self.atomic_value.load(Ordering::Acquire);
        if current != expected {
            return Ok(()); // Value changed, no need to wait
        }

        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref vxworks_sem) = self.vxworks_sem {
                extern "C" {
                    fn semTake(sem_id: usize, timeout: i32) -> i32;
                }

                let timeout_ticks = timeout.map_or(-1, Self::duration_to_ticks); // -1 = WAIT_FOREVER
                let result = unsafe { semTake(vxworks_sem.sem_id, timeout_ticks) };
                
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "VxWorks LKM semTake failed"
                    ));
                }
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation - just simulate a brief wait
            if let Some(duration) = timeout {
                if duration.as_millis() > 0 {
                    // In real implementation, would actually wait
                }
            }
        }

        Ok(())
    }

    fn wake_one(&self) -> Result<u32, Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref vxworks_sem) = self.vxworks_sem {
                extern "C" {
                    fn semGive(sem_id: usize) -> i32;
                }

                let result = unsafe { semGive(vxworks_sem.sem_id) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "VxWorks LKM semGive failed"
                    ));
                }
                return Ok(1); // Woke one task
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation
            return Ok(1);
        }

        Ok(0)
    }

    fn wake_all(&self) -> Result<u32, Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref vxworks_sem) = self.vxworks_sem {
                extern "C" {
                    fn semFlush(sem_id: usize) -> i32;
                }

                let result = unsafe { semFlush(vxworks_sem.sem_id) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "VxWorks LKM semFlush failed"
                    ));
                }
                return Ok(u32::MAX); // Indicate potentially many tasks woken
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation
            return Ok(8); // Simulate waking multiple tasks
        }

        Ok(0)
    }

    fn load(&self, ordering: Ordering) -> u32 {
        self.atomic_value.load(ordering)
    }

    fn store(&self, value: u32, ordering: Ordering) {
        self.atomic_value.store(value, ordering);
    }

    fn compare_exchange_weak(
        &self,
        current: u32,
        new: u32,
        success: Ordering,
        failure: Ordering,
    ) -> Result<u32, u32> {
        self.atomic_value.compare_exchange_weak(current, new, success, failure)
    }
}

impl Drop for VxWorksLkmFutex {
    fn drop(&mut self) {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(ref vxworks_sem) = self.vxworks_sem {
                extern "C" {
                    fn semDelete(sem_id: usize) -> i32;
                }
                
                unsafe {
                    semDelete(vxworks_sem.sem_id);
                }
            }
        }
    }
}

/// Builder for VxWorks RTP futex
pub struct VxWorksRtpFutexBuilder {
    initial_value: u32,
    priority_inheritance: bool,
}

impl VxWorksRtpFutexBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            initial_value: 0,
            priority_inheritance: true,
        }
    }

    /// Set initial value
    pub fn with_initial_value(mut self, value: u32) -> Self {
        self.initial_value = value;
        self
    }

    /// Enable priority inheritance
    pub fn with_priority_inheritance(mut self, enable: bool) -> Self {
        self.priority_inheritance = enable;
        self
    }

    /// Build the futex
    pub fn build(self) -> Result<VxWorksRtpFutex, Error> {
        let mut futex = VxWorksRtpFutex {
            atomic_value: AtomicU32::new(self.initial_value),
            priority_inheritance: self.priority_inheritance,
            posix_sem: None,
        };

        futex.init_posix_semaphore(self.initial_value)?;
        Ok(futex)
    }
}

impl Default for VxWorksRtpFutexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for VxWorks LKM futex
pub struct VxWorksLkmFutexBuilder {
    initial_value: u32,
    priority_inheritance: bool,
}

impl VxWorksLkmFutexBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            initial_value: 0,
            priority_inheritance: true,
        }
    }

    /// Set initial value
    pub fn with_initial_value(mut self, value: u32) -> Self {
        self.initial_value = value;
        self
    }

    /// Enable priority inheritance
    pub fn with_priority_inheritance(mut self, enable: bool) -> Self {
        self.priority_inheritance = enable;
        self
    }

    /// Build the futex
    pub fn build(self) -> Result<VxWorksLkmFutex, Error> {
        let mut futex = VxWorksLkmFutex {
            atomic_value: AtomicU32::new(self.initial_value),
            priority_inheritance: self.priority_inheritance,
            vxworks_sem: None,
        };

        futex.init_vxworks_semaphore()?;
        Ok(futex)
    }
}

impl Default for VxWorksLkmFutexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_futex_builder() {
        let futex = VxWorksRtpFutex::new()
            .with_initial_value(42)
            .with_priority_inheritance(false)
            .build()
            .unwrap();

        assert_eq!(futex.load(Ordering::Relaxed), 42);
        assert!(!futex.priority_inheritance);
    }

    #[test]
    fn test_lkm_futex_builder() {
        let futex = VxWorksLkmFutex::new()
            .with_initial_value(123)
            .with_priority_inheritance(true)
            .build()
            .unwrap();

        assert_eq!(futex.load(Ordering::Relaxed), 123);
        assert!(futex.priority_inheritance);
    }

    #[test]
    fn test_atomic_operations() {
        let futex = VxWorksRtpFutex::new()
            .with_initial_value(10)
            .build()
            .unwrap();

        // Test store/load
        futex.store(20, Ordering::SeqCst);
        assert_eq!(futex.load(Ordering::SeqCst), 20);

        // Test compare_exchange_weak
        let result = futex.compare_exchange_weak(20, 30, Ordering::SeqCst, Ordering::SeqCst);
        assert_eq!(result, Ok(20));
        assert_eq!(futex.load(Ordering::SeqCst), 30);
    }

    #[cfg(not(target_os = "vxworks"))]
    #[test]
    fn test_futex_operations() {
        let futex = VxWorksRtpFutex::new()
            .build()
            .unwrap();

        // Test wake operations (mock implementation)
        assert_eq!(futex.wake_one().unwrap(), 1);
        assert!(futex.wake_all().unwrap() > 0);

        // Test wait with immediate return (value mismatch)
        assert!(futex.wait(999, Some(Duration::from_millis(1))).is_ok());
    }

    #[test]
    fn test_duration_conversion() {
        let duration = Duration::from_millis(100);
        let ticks = VxWorksLkmFutex::duration_to_ticks(duration);
        
        #[cfg(target_os = "vxworks")]
        assert!(ticks > 0);
        
        #[cfg(not(target_os = "vxworks"))]
        assert_eq!(ticks, 100);
    }
}