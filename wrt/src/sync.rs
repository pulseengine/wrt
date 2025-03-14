// We'll use () as the error type for now

/// A simple mutex implementation for no_std environments
///
/// This is a placeholder implementation that doesn't actually
/// provide thread safety. In a real implementation, we'd use
/// a proper no_std mutex like spin::Mutex.
#[derive(Debug)]
pub struct Mutex<T> {
    inner: core::cell::UnsafeCell<T>,
}

/// Guard for the mutex that provides access to the inner value
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    /// Create a new mutex
    pub fn new(value: T) -> Self {
        Self {
            inner: core::cell::UnsafeCell::new(value),
        }
    }

    /// Lock the mutex and get access to the inner value
    #[allow(clippy::result_unit_err)]
    pub fn lock(&self) -> Result<MutexGuard<'_, T>, ()> {
        // In a real implementation, we'd actually lock here
        Ok(MutexGuard { mutex: self })
    }
}

impl<T> core::ops::Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

// These are required for Send and Sync implementations
// Note: This is not thread-safe in a no_std environment without actual locking
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}
