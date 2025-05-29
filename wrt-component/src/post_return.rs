//! Post-return cleanup functions for WebAssembly Component Model
//!
//! This module implements the post-return cleanup mechanism that allows
//! components to perform cleanup after function calls, particularly for
//! managing resources and memory allocations.

#[cfg(feature = "std")]
use std::sync::{Arc, RwLock, Mutex};
#[cfg(not(feature = "std"))]
use alloc::{sync::{Arc, Mutex}, boxed::Box};

use wrt_foundation::{
    bounded_collections::{BoundedVec, BoundedString, MAX_GENERATIVE_TYPES},
    prelude::*,
};

use crate::{
    types::{ComponentError, ComponentInstanceId, TypeId},
    canonical_realloc::ReallocManager,
    component_resolver::ComponentValue,
};

/// Post-return function signature: () -> ()
pub type PostReturnFn = fn();

/// Post-return cleanup registry
#[derive(Debug)]
pub struct PostReturnRegistry {
    /// Registered post-return functions per instance
    functions: BTreeMap<ComponentInstanceId, PostReturnFunction>,
    /// Cleanup tasks waiting to be executed
    pending_cleanups: BTreeMap<ComponentInstanceId, BoundedVec<CleanupTask, MAX_GENERATIVE_TYPES>>,
    /// Execution metrics
    metrics: PostReturnMetrics,
    /// Maximum cleanup tasks per instance
    max_cleanup_tasks: usize,
}

#[derive(Debug, Clone)]
struct PostReturnFunction {
    /// Function index in the component
    func_index: u32,
    /// Cached function reference for performance
    func_ref: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether the function is currently being executed
    executing: bool,
}

#[derive(Debug, Clone)]
pub struct CleanupTask {
    /// Type of cleanup task
    pub task_type: CleanupTaskType,
    /// Instance that created this task
    pub source_instance: ComponentInstanceId,
    /// Priority of the task (higher = execute first)
    pub priority: u8,
    /// Task-specific data
    pub data: CleanupData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CleanupTaskType {
    /// Deallocate memory
    DeallocateMemory,
    /// Close resource handle
    CloseResource,
    /// Release reference
    ReleaseReference,
    /// Custom cleanup
    Custom,
    /// Async cleanup (for streams/futures)
    AsyncCleanup,
}

#[derive(Debug, Clone)]
pub enum CleanupData {
    /// Memory deallocation data
    Memory {
        ptr: i32,
        size: i32,
        align: i32,
    },
    /// Resource cleanup data
    Resource {
        handle: u32,
        resource_type: TypeId,
    },
    /// Reference cleanup data
    Reference {
        ref_id: u32,
        ref_count: u32,
    },
    /// Custom cleanup data
    Custom {
        cleanup_id: BoundedString<64>,
        parameters: BoundedVec<ComponentValue, 16>,
    },
    /// Async cleanup data
    Async {
        stream_handle: Option<u32>,
        future_handle: Option<u32>,
        task_id: Option<u32>,
    },
}

#[derive(Debug, Default, Clone)]
struct PostReturnMetrics {
    /// Total post-return functions executed
    total_executions: u64,
    /// Total cleanup tasks processed
    total_cleanup_tasks: u64,
    /// Failed cleanup attempts
    failed_cleanups: u64,
    /// Average cleanup time (microseconds)
    avg_cleanup_time_us: u64,
    /// Peak pending cleanup tasks
    peak_pending_tasks: usize,
}

/// Context for post-return execution
pub struct PostReturnContext {
    /// Instance being cleaned up
    pub instance_id: ComponentInstanceId,
    /// Cleanup tasks to execute
    pub tasks: Vec<CleanupTask>,
    /// Realloc manager for memory cleanup
    pub realloc_manager: Option<Arc<RwLock<ReallocManager>>>,
    /// Custom cleanup handlers
    pub custom_handlers: BTreeMap<BoundedString<64>, Box<dyn Fn(&CleanupData) -> Result<(), ComponentError> + Send + Sync>>,
}

impl PostReturnRegistry {
    pub fn new(max_cleanup_tasks: usize) -> Self {
        Self {
            functions: BTreeMap::new(),
            pending_cleanups: BTreeMap::new(),
            metrics: PostReturnMetrics::default(),
            max_cleanup_tasks,
        }
    }

    /// Register a post-return function for an instance
    pub fn register_post_return(
        &mut self,
        instance_id: ComponentInstanceId,
        func_index: u32,
    ) -> Result<(), ComponentError> {
        let post_return_fn = PostReturnFunction {
            func_index,
            func_ref: None,
            executing: false,
        };

        self.functions.insert(instance_id, post_return_fn);
        self.pending_cleanups.insert(instance_id, BoundedVec::new());

        Ok(())
    }

    /// Schedule a cleanup task to be executed during post-return
    pub fn schedule_cleanup(
        &mut self,
        instance_id: ComponentInstanceId,
        task: CleanupTask,
    ) -> Result<(), ComponentError> {
        let cleanup_tasks = self.pending_cleanups
            .get_mut(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        if cleanup_tasks.len() >= self.max_cleanup_tasks {
            return Err(ComponentError::TooManyGenerativeTypes);
        }

        cleanup_tasks.push(task)
            .map_err(|_| ComponentError::TooManyGenerativeTypes)?;

        // Update peak tasks metric
        let total_pending = self.pending_cleanups
            .values()
            .map(|tasks| tasks.len())
            .sum();
        
        if total_pending > self.metrics.peak_pending_tasks {
            self.metrics.peak_pending_tasks = total_pending;
        }

        Ok(())
    }

    /// Execute post-return cleanup for an instance
    pub fn execute_post_return(
        &mut self,
        instance_id: ComponentInstanceId,
        context: PostReturnContext,
    ) -> Result<(), ComponentError> {
        // Check if post-return function exists and isn't already executing
        let post_return_fn = self.functions
            .get_mut(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        if post_return_fn.executing {
            return Err(ComponentError::ResourceNotFound(0)); // Already executing
        }

        post_return_fn.executing = true;

        let start_time = std::time::Instant::now();
        let result = self.execute_cleanup_tasks(instance_id, context);
        let elapsed = start_time.elapsed().as_micros() as u64;

        // Update metrics
        self.metrics.total_executions += 1;
        if result.is_err() {
            self.metrics.failed_cleanups += 1;
        }

        // Update average cleanup time
        self.metrics.avg_cleanup_time_us = 
            (self.metrics.avg_cleanup_time_us + elapsed) / 2;

        post_return_fn.executing = false;

        // Clear pending cleanups
        if let Some(cleanup_tasks) = self.pending_cleanups.get_mut(&instance_id) {
            cleanup_tasks.clear();
        }

        result
    }

    /// Execute all pending cleanup tasks
    fn execute_cleanup_tasks(
        &mut self,
        instance_id: ComponentInstanceId,
        mut context: PostReturnContext,
    ) -> Result<(), ComponentError> {
        // Get all pending cleanup tasks
        let mut all_tasks = context.tasks;
        
        if let Some(pending) = self.pending_cleanups.get(&instance_id) {
            all_tasks.extend(pending.iter().cloned());
        }

        // Sort tasks by priority (highest first)
        all_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Execute each task
        for task in all_tasks {
            self.execute_single_cleanup_task(&task, &mut context)?;
            self.metrics.total_cleanup_tasks += 1;
        }

        Ok(())
    }

    /// Execute a single cleanup task
    fn execute_single_cleanup_task(
        &self,
        task: &CleanupTask,
        context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        match task.task_type {
            CleanupTaskType::DeallocateMemory => {
                self.cleanup_memory(task, context)
            }
            CleanupTaskType::CloseResource => {
                self.cleanup_resource(task, context)
            }
            CleanupTaskType::ReleaseReference => {
                self.cleanup_reference(task, context)
            }
            CleanupTaskType::Custom => {
                self.cleanup_custom(task, context)
            }
            CleanupTaskType::AsyncCleanup => {
                self.cleanup_async(task, context)
            }
        }
    }

    /// Clean up memory allocation
    fn cleanup_memory(
        &self,
        task: &CleanupTask,
        context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        if let CleanupData::Memory { ptr, size, align } = &task.data {
            if let Some(realloc_manager) = &context.realloc_manager {
                let mut manager = realloc_manager.write()
                    .map_err(|_| ComponentError::ResourceNotFound(0))?;
                
                manager.deallocate(task.source_instance, *ptr, *size, *align)?;
            }
        }
        Ok(())
    }

    /// Clean up resource handle
    fn cleanup_resource(
        &self,
        task: &CleanupTask,
        _context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        if let CleanupData::Resource { handle, resource_type } = &task.data {
            // In a real implementation, this would call resource destructors
            // For now, we just acknowledge the cleanup
            Ok(())
        } else {
            Err(ComponentError::TypeMismatch)
        }
    }

    /// Clean up reference count
    fn cleanup_reference(
        &self,
        task: &CleanupTask,
        _context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        if let CleanupData::Reference { ref_id, ref_count } = &task.data {
            // Decrement reference count and potentially deallocate
            // Implementation would depend on reference counting system
            Ok(())
        } else {
            Err(ComponentError::TypeMismatch)
        }
    }

    /// Execute custom cleanup
    fn cleanup_custom(
        &self,
        task: &CleanupTask,
        context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        if let CleanupData::Custom { cleanup_id, parameters: _ } = &task.data {
            if let Some(handler) = context.custom_handlers.get(cleanup_id) {
                handler(&task.data)?;
            }
        }
        Ok(())
    }

    /// Clean up async resources
    fn cleanup_async(
        &self,
        task: &CleanupTask,
        _context: &mut PostReturnContext,
    ) -> Result<(), ComponentError> {
        if let CleanupData::Async { stream_handle, future_handle, task_id } = &task.data {
            // Cancel/cleanup async operations
            // This would integrate with the async system
            Ok(())
        } else {
            Err(ComponentError::TypeMismatch)
        }
    }

    /// Remove all cleanup tasks for an instance
    pub fn cleanup_instance(&mut self, instance_id: ComponentInstanceId) -> Result<(), ComponentError> {
        self.functions.remove(&instance_id);
        self.pending_cleanups.remove(&instance_id);
        Ok(())
    }

    /// Get execution metrics
    pub fn metrics(&self) -> &PostReturnMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = PostReturnMetrics::default();
    }
}

/// Helper functions for common cleanup scenarios
pub mod helpers {
    use super::*;

    /// Create a memory cleanup task
    pub fn memory_cleanup_task(
        instance_id: ComponentInstanceId,
        ptr: i32,
        size: i32,
        align: i32,
        priority: u8,
    ) -> CleanupTask {
        CleanupTask {
            task_type: CleanupTaskType::DeallocateMemory,
            source_instance: instance_id,
            priority,
            data: CleanupData::Memory { ptr, size, align },
        }
    }

    /// Create a resource cleanup task
    pub fn resource_cleanup_task(
        instance_id: ComponentInstanceId,
        handle: u32,
        resource_type: TypeId,
        priority: u8,
    ) -> CleanupTask {
        CleanupTask {
            task_type: CleanupTaskType::CloseResource,
            source_instance: instance_id,
            priority,
            data: CleanupData::Resource { handle, resource_type },
        }
    }

    /// Create an async cleanup task
    pub fn async_cleanup_task(
        instance_id: ComponentInstanceId,
        stream_handle: Option<u32>,
        future_handle: Option<u32>,
        task_id: Option<u32>,
        priority: u8,
    ) -> CleanupTask {
        CleanupTask {
            task_type: CleanupTaskType::AsyncCleanup,
            source_instance: instance_id,
            priority,
            data: CleanupData::Async { stream_handle, future_handle, task_id },
        }
    }

    /// Create a custom cleanup task
    pub fn custom_cleanup_task(
        instance_id: ComponentInstanceId,
        cleanup_id: &str,
        parameters: Vec<ComponentValue>,
        priority: u8,
    ) -> Result<CleanupTask, ComponentError> {
        let cleanup_id = BoundedString::from_str(cleanup_id)
            .map_err(|_| ComponentError::TypeMismatch)?;
        
        let mut param_vec = BoundedVec::new();
        for param in parameters {
            param_vec.push(param)
                .map_err(|_| ComponentError::TooManyGenerativeTypes)?;
        }

        Ok(CleanupTask {
            task_type: CleanupTaskType::Custom,
            source_instance: instance_id,
            priority,
            data: CleanupData::Custom {
                cleanup_id,
                parameters: param_vec,
            },
        })
    }
}

impl Default for PostReturnRegistry {
    fn default() -> Self {
        Self::new(1024) // Default max 1024 cleanup tasks per instance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_realloc::ReallocManager;

    #[test]
    fn test_post_return_registry_creation() {
        let registry = PostReturnRegistry::new(100);
        assert_eq!(registry.max_cleanup_tasks, 100);
        assert_eq!(registry.functions.len(), 0);
    }

    #[test]
    fn test_register_post_return() {
        let mut registry = PostReturnRegistry::new(100);
        let instance_id = ComponentInstanceId(1);

        assert!(registry.register_post_return(instance_id, 42).is_ok());
        assert!(registry.functions.contains_key(&instance_id));
    }

    #[test]
    fn test_schedule_cleanup() {
        let mut registry = PostReturnRegistry::new(100);
        let instance_id = ComponentInstanceId(1);

        registry.register_post_return(instance_id, 42).unwrap();

        let task = helpers::memory_cleanup_task(instance_id, 0x1000, 64, 8, 10);
        assert!(registry.schedule_cleanup(instance_id, task).is_ok());

        assert_eq!(registry.pending_cleanups[&instance_id].len(), 1);
    }

    #[test]
    fn test_cleanup_task_helpers() {
        let instance_id = ComponentInstanceId(1);

        // Test memory cleanup task
        let mem_task = helpers::memory_cleanup_task(instance_id, 0x1000, 64, 8, 10);
        assert_eq!(mem_task.task_type, CleanupTaskType::DeallocateMemory);
        assert_eq!(mem_task.priority, 10);

        // Test resource cleanup task
        let res_task = helpers::resource_cleanup_task(instance_id, 42, TypeId(1), 5);
        assert_eq!(res_task.task_type, CleanupTaskType::CloseResource);
        assert_eq!(res_task.priority, 5);

        // Test async cleanup task
        let async_task = helpers::async_cleanup_task(instance_id, Some(1), Some(2), Some(3), 8);
        assert_eq!(async_task.task_type, CleanupTaskType::AsyncCleanup);
        assert_eq!(async_task.priority, 8);

        // Test custom cleanup task
        let custom_task = helpers::custom_cleanup_task(
            instance_id,
            "custom_cleanup",
            vec![ComponentValue::U32(42)],
            7,
        );
        assert!(custom_task.is_ok());
        let custom_task = custom_task.unwrap();
        assert_eq!(custom_task.task_type, CleanupTaskType::Custom);
        assert_eq!(custom_task.priority, 7);
    }

    #[test]
    fn test_cleanup_task_limits() {
        let mut registry = PostReturnRegistry::new(2); // Small limit for testing
        let instance_id = ComponentInstanceId(1);

        registry.register_post_return(instance_id, 42).unwrap();

        // Add tasks up to limit
        let task1 = helpers::memory_cleanup_task(instance_id, 0x1000, 64, 8, 10);
        let task2 = helpers::memory_cleanup_task(instance_id, 0x2000, 64, 8, 10);
        let task3 = helpers::memory_cleanup_task(instance_id, 0x3000, 64, 8, 10);

        assert!(registry.schedule_cleanup(instance_id, task1).is_ok());
        assert!(registry.schedule_cleanup(instance_id, task2).is_ok());
        assert!(registry.schedule_cleanup(instance_id, task3).is_err()); // Should fail
    }

    #[test]
    fn test_metrics() {
        let registry = PostReturnRegistry::new(100);
        let metrics = registry.metrics();
        
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.total_cleanup_tasks, 0);
        assert_eq!(metrics.failed_cleanups, 0);
    }
}