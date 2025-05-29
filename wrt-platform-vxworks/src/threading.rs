//! VxWorks-specific threading support
//!
//! This module provides external implementations of threading primitives
//! for VxWorks that work with both RTP and LKM contexts.

use core::ffi::c_void;
use wrt_error::{Error, ErrorKind};
use crate::ExecutionContext;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// VxWorks task priority (0-255, lower is higher priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(u8);

impl TaskPriority {
    /// Create a new task priority
    pub fn new(priority: u8) -> Self {
        Self(priority)
    }

    /// Get the priority value
    pub fn value(self) -> u8 {
        self.0
    }

    /// Highest priority (0)
    pub const HIGHEST: Self = Self(0);
    
    /// High priority (50)
    pub const HIGH: Self = Self(50);
    
    /// Normal priority (100)
    pub const NORMAL: Self = Self(100);
    
    /// Low priority (150)
    pub const LOW: Self = Self(150);
    
    /// Lowest priority (255)
    pub const LOWEST: Self = Self(255);
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// VxWorks task handle (for LKM context)
pub struct VxWorksTask {
    task_id: Option<usize>,
    #[cfg(feature = "alloc")]
    name: Option<String>,
    priority: TaskPriority,
    stack_size: usize,
}

/// VxWorks thread handle (for RTP context)  
pub struct VxWorksThread {
    #[cfg(target_os = "vxworks")]
    pthread: Option<PThread>,
    #[cfg(not(target_os = "vxworks"))]
    thread_id: Option<u64>,
    #[cfg(feature = "alloc")]
    name: Option<String>,
    stack_size: usize,
    detached: bool,
}

#[cfg(target_os = "vxworks")]
#[repr(C)]
struct PThread {
    _data: [u8; 8], // Platform-specific thread handle
}

#[cfg(target_os = "vxworks")]
#[repr(C)]
struct PThreadAttr {
    _data: [u8; 32], // Platform-specific thread attributes
}

/// Thread entry point function type
#[cfg(feature = "alloc")]
pub type ThreadEntryPoint = Box<dyn FnOnce() + Send + 'static>;

/// VxWorks task configuration
#[derive(Debug, Clone)]
pub struct VxWorksTaskConfig {
    pub context: ExecutionContext,
    pub stack_size: usize,
    pub priority: TaskPriority,
    #[cfg(feature = "alloc")]
    pub name: Option<String>,
    pub floating_point: bool,
    pub detached: bool,
}

impl Default for VxWorksTaskConfig {
    fn default() -> Self {
        Self {
            context: ExecutionContext::Rtp,
            stack_size: 64 * 1024, // 64KB
            priority: TaskPriority::NORMAL,
            #[cfg(feature = "alloc")]
            name: None,
            floating_point: false,
            detached: false,
        }
    }
}

impl VxWorksTask {
    /// Spawn a new VxWorks task (LKM context)
    #[cfg(feature = "alloc")]
    pub fn spawn<F>(config: VxWorksTaskConfig, f: F) -> Result<Self, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        if config.context != ExecutionContext::Lkm {
            return Err(Error::new(
                ErrorKind::Platform,
                "VxWorks tasks are only supported in LKM context"
            ));
        }

        let entry_point = Box::new(f);
        
        #[cfg(target_os = "vxworks")]
        {
            extern "C" fn task_wrapper(arg: *mut c_void) -> i32 {
                let closure: Box<ThreadEntryPoint> = unsafe { Box::from_raw(arg as *mut ThreadEntryPoint) };
                closure();
                0 // Success
            }

            extern "C" {
                fn taskSpawn(
                    name: *const u8,
                    priority: i32,
                    options: i32,
                    stack_size: usize,
                    entry_point: extern "C" fn(*mut c_void) -> i32,
                    arg1: *mut c_void,
                    arg2: usize, arg3: usize, arg4: usize, arg5: usize,
                    arg6: usize, arg7: usize, arg8: usize, arg9: usize, arg10: usize,
                ) -> usize;
            }

            let closure_ptr = Box::into_raw(entry_point) as *mut c_void;
            let name_ptr = config.name.as_ref()
                .map(|n| n.as_ptr())
                .unwrap_or(b"wrt_task\0".as_ptr());
            
            let priority = config.priority.value() as i32;
            let mut options = 0;
            
            // VxWorks task options
            const VX_FP_TASK: i32 = 0x0008;
            if config.floating_point {
                options |= VX_FP_TASK;
            }
            
            let task_id = unsafe {
                taskSpawn(
                    name_ptr,
                    priority,
                    options,
                    config.stack_size,
                    task_wrapper,
                    closure_ptr,
                    0, 0, 0, 0, 0, 0, 0, 0, 0,
                )
            };

            if task_id == 0 {
                // Cleanup on failure
                unsafe { let _ = Box::from_raw(closure_ptr as *mut ThreadEntryPoint); }
                return Err(Error::new(
                    ErrorKind::Platform,
                    "Failed to spawn VxWorks task"
                ));
            }

            Ok(Self {
                task_id: Some(task_id),
                name: config.name,
                priority: config.priority,
                stack_size: config.stack_size,
            })
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            // In a real scenario, this would not be used
            drop(entry_point); // Prevent unused variable warning
            
            Ok(Self {
                task_id: Some(1), // Mock task ID
                name: config.name,
                priority: config.priority,
                stack_size: config.stack_size,
            })
        }
    }

    /// Get the task ID
    pub fn task_id(&self) -> Option<usize> {
        self.task_id
    }

    /// Get the task priority
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    /// Set task priority
    pub fn set_priority(&mut self, priority: TaskPriority) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(task_id) = self.task_id {
                extern "C" {
                    fn taskPrioritySet(task_id: usize, new_priority: i32) -> i32;
                }

                let result = unsafe { taskPrioritySet(task_id, priority.value() as i32) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "Failed to set VxWorks task priority"
                    ));
                }
            }
        }
        
        self.priority = priority;
        Ok(())
    }

    /// Suspend the task
    pub fn suspend(&self) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(task_id) = self.task_id {
                extern "C" {
                    fn taskSuspend(task_id: usize) -> i32;
                }

                let result = unsafe { taskSuspend(task_id) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "Failed to suspend VxWorks task"
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Resume the task
    pub fn resume(&self) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(task_id) = self.task_id {
                extern "C" {
                    fn taskResume(task_id: usize) -> i32;
                }

                let result = unsafe { taskResume(task_id) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "Failed to resume VxWorks task"
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Get the current task ID
    pub fn current_task_id() -> usize {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn taskIdSelf() -> usize;
            }
            
            unsafe { taskIdSelf() }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            1 // Mock task ID
        }
    }

    /// Delay the current task
    pub fn delay_ms(milliseconds: u32) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn taskDelay(ticks: i32) -> i32;
                fn sysClkRateGet() -> i32;
            }

            let ticks_per_sec = unsafe { sysClkRateGet() } as u32;
            let ticks = (milliseconds * ticks_per_sec) / 1000;
            
            let result = unsafe { taskDelay(ticks as i32) };
            if result != 0 {
                return Err(Error::new(
                    ErrorKind::Platform,
                    "VxWorks task delay failed"
                ));
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation - do nothing
            let _ = milliseconds;
        }
        
        Ok(())
    }
}

impl Drop for VxWorksTask {
    fn drop(&mut self) {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(task_id) = self.task_id {
                extern "C" {
                    fn taskDelete(task_id: usize) -> i32;
                }
                
                unsafe {
                    taskDelete(task_id);
                }
            }
        }
    }
}

impl VxWorksThread {
    /// Spawn a new VxWorks thread (RTP context)
    #[cfg(feature = "alloc")]
    pub fn spawn<F>(config: VxWorksTaskConfig, f: F) -> Result<Self, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        if config.context != ExecutionContext::Rtp {
            return Err(Error::new(
                ErrorKind::Platform,
                "VxWorks threads are only supported in RTP context"
            ));
        }

        let entry_point = Box::new(f);
        
        #[cfg(target_os = "vxworks")]
        {
            extern "C" fn thread_wrapper(arg: *mut c_void) -> *mut c_void {
                let closure: Box<ThreadEntryPoint> = unsafe { Box::from_raw(arg as *mut ThreadEntryPoint) };
                closure();
                core::ptr::null_mut()
            }

            extern "C" {
                fn pthread_create(
                    thread: *mut PThread,
                    attr: *const PThreadAttr,
                    start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
                    arg: *mut c_void,
                ) -> i32;
                fn pthread_attr_init(attr: *mut PThreadAttr) -> i32;
                fn pthread_attr_destroy(attr: *mut PThreadAttr) -> i32;
                fn pthread_attr_setstacksize(attr: *mut PThreadAttr, stacksize: usize) -> i32;
                fn pthread_attr_setdetachstate(attr: *mut PThreadAttr, detachstate: i32) -> i32;
            }

            let closure_ptr = Box::into_raw(entry_point) as *mut c_void;
            
            // Initialize thread attributes
            let mut attr = PThreadAttr { _data: [0; 32] };
            let attr_result = unsafe { pthread_attr_init(&mut attr) };
            if attr_result != 0 {
                unsafe { let _ = Box::from_raw(closure_ptr as *mut ThreadEntryPoint); }
                return Err(Error::new(
                    ErrorKind::Platform,
                    "Failed to initialize pthread attributes"
                ));
            }

            // Set stack size
            unsafe {
                pthread_attr_setstacksize(&mut attr, config.stack_size);
            }

            // Set detach state
            const PTHREAD_CREATE_DETACHED: i32 = 1;
            if config.detached {
                unsafe {
                    pthread_attr_setdetachstate(&mut attr, PTHREAD_CREATE_DETACHED);
                }
            }

            // Create the thread
            let mut pthread = PThread { _data: [0; 8] };
            let create_result = unsafe {
                pthread_create(&mut pthread, &attr, thread_wrapper, closure_ptr)
            };

            // Clean up attributes
            unsafe { pthread_attr_destroy(&mut attr); }

            if create_result != 0 {
                unsafe { let _ = Box::from_raw(closure_ptr as *mut ThreadEntryPoint); }
                return Err(Error::new(
                    ErrorKind::Platform,
                    "Failed to create pthread"
                ));
            }

            Ok(Self {
                pthread: Some(pthread),
                name: config.name,
                stack_size: config.stack_size,
                detached: config.detached,
            })
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            drop(entry_point); // Prevent unused variable warning
            
            Ok(Self {
                thread_id: Some(1), // Mock thread ID
                name: config.name,
                stack_size: config.stack_size,
                detached: config.detached,
            })
        }
    }

    /// Join the thread (wait for completion)
    pub fn join(self) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(pthread) = self.pthread {
                extern "C" {
                    fn pthread_join(thread: PThread, retval: *mut *mut c_void) -> i32;
                }

                let result = unsafe { pthread_join(pthread, core::ptr::null_mut()) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "Failed to join pthread"
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Detach the thread
    pub fn detach(self) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            if let Some(pthread) = self.pthread {
                extern "C" {
                    fn pthread_detach(thread: PThread) -> i32;
                }

                let result = unsafe { pthread_detach(pthread) };
                if result != 0 {
                    return Err(Error::new(
                        ErrorKind::Platform,
                        "Failed to detach pthread"
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Get current pthread ID
    pub fn current_thread_id() -> u64 {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn pthread_self() -> PThread;
            }
            
            let pthread = unsafe { pthread_self() };
            unsafe { core::mem::transmute::<PThread, u64>(pthread) }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            1 // Mock thread ID
        }
    }

    /// Yield the current thread
    pub fn yield_now() -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn pthread_yield() -> i32;
            }
            
            unsafe { pthread_yield(); }
        }
        
        Ok(())
    }
}

/// Builder for VxWorks tasks and threads
pub struct VxWorksTaskBuilder {
    config: VxWorksTaskConfig,
}

impl VxWorksTaskBuilder {
    /// Create new builder
    pub fn new(context: ExecutionContext) -> Self {
        Self {
            config: VxWorksTaskConfig {
                context,
                ..Default::default()
            },
        }
    }

    /// Set stack size
    pub fn stack_size(mut self, size: usize) -> Self {
        self.config.stack_size = size;
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.config.priority = priority;
        self
    }

    /// Set name
    #[cfg(feature = "alloc")]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.name = Some(name.into());
        self
    }

    /// Enable floating point
    pub fn floating_point(mut self, enable: bool) -> Self {
        self.config.floating_point = enable;
        self
    }

    /// Set detached mode
    pub fn detached(mut self, detached: bool) -> Self {
        self.config.detached = detached;
        self
    }

    /// Spawn task for LKM context
    #[cfg(feature = "alloc")]
    pub fn spawn_task<F>(self, f: F) -> Result<VxWorksTask, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        VxWorksTask::spawn(self.config, f)
    }

    /// Spawn thread for RTP context
    #[cfg(feature = "alloc")]
    pub fn spawn_thread<F>(self, f: F) -> Result<VxWorksThread, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        VxWorksThread::spawn(self.config, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority() {
        let priority = TaskPriority::new(100);
        assert_eq!(priority.value(), 100);
        
        assert_eq!(TaskPriority::HIGHEST.value(), 0);
        assert_eq!(TaskPriority::NORMAL.value(), 100);
        assert_eq!(TaskPriority::LOWEST.value(), 255);
        
        assert!(TaskPriority::HIGHEST < TaskPriority::NORMAL);
        assert!(TaskPriority::NORMAL < TaskPriority::LOWEST);
    }

    #[test]
    fn test_task_config() {
        let config = VxWorksTaskConfig {
            context: ExecutionContext::Lkm,
            stack_size: 128 * 1024,
            priority: TaskPriority::HIGH,
            #[cfg(feature = "alloc")]
            name: Some("test_task".to_string()),
            floating_point: true,
            detached: false,
        };

        assert_eq!(config.context, ExecutionContext::Lkm);
        assert_eq!(config.stack_size, 128 * 1024);
        assert_eq!(config.priority, TaskPriority::HIGH);
        assert!(config.floating_point);
        assert!(!config.detached);
    }

    #[test]
    fn test_task_builder() {
        let builder = VxWorksTaskBuilder::new(ExecutionContext::Lkm)
            .stack_size(256 * 1024)
            .priority(TaskPriority::LOW)
            .floating_point(true)
            .detached(true);

        assert_eq!(builder.config.context, ExecutionContext::Lkm);
        assert_eq!(builder.config.stack_size, 256 * 1024);
        assert_eq!(builder.config.priority, TaskPriority::LOW);
        assert!(builder.config.floating_point);
        assert!(builder.config.detached);
    }

    #[test]
    fn test_current_ids() {
        let task_id = VxWorksTask::current_task_id();
        let thread_id = VxWorksThread::current_thread_id();
        
        #[cfg(target_os = "vxworks")]
        {
            assert!(task_id > 0);
            assert!(thread_id > 0);
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            assert_eq!(task_id, 1);
            assert_eq!(thread_id, 1);
        }
    }
}