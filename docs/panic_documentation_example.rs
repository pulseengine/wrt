//! Example module showing proper panic documentation patterns.

/// A bounded vector with a maximum capacity.
pub struct BoundedVec<T> {
    /// The inner vector storing elements
    inner: Vec<T>,
    /// Maximum capacity of the vector
    capacity: usize,
}

impl<T> BoundedVec<T> {
    /// Creates a new bounded vector with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        BoundedVec {
            inner: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Pushes an element to the vector.
    /// 
    /// # Panics
    /// This function will panic if the vector has reached its capacity.
    /// Safety impact: MEDIUM
    /// Tracking: WRTQ-400
    pub fn push(&mut self, value: T) {
        if self.inner.len() >= self.capacity {
            panic!("BoundedVec capacity exceeded");
        }
        self.inner.push(value);
    }
    
    /// Gets an element at the specified index.
    /// 
    /// # Panics
    /// This function will panic if the index is out of bounds.
    /// Safety impact: MEDIUM
    /// Tracking: WRTQ-401
    pub fn get(&self, index: usize) -> &T {
        &self.inner[index] // This will panic if index is out of bounds
    }
    
    /// Returns the current length of the vector.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    /// Returns whether the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Attempts to push an element to the vector, returning an error if at capacity.
    /// This is a safer alternative to the `push` method.
    pub fn try_push(&mut self, value: T) -> Result<(), &'static str> {
        if self.inner.len() >= self.capacity {
            return Err("BoundedVec capacity exceeded");
        }
        self.inner.push(value);
        Ok(())
    }
    
    /// Attempts to get an element at the specified index.
    /// This is a safer alternative to the `get` method.
    pub fn try_get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }
}

/// A thread-safe mutex implementation.
pub struct Mutex<T> {
    inner: std::sync::Mutex<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub fn new(value: T) -> Self {
        Mutex {
            inner: std::sync::Mutex::new(value),
        }
    }
    
    /// Acquires the lock and returns a guard.
    /// 
    /// # Panics
    /// This function will panic if another thread holding the lock panicked while holding the lock.
    /// This can lead to poisoned mutex state.
    /// Safety impact: HIGH
    /// Tracking: WRTQ-300
    pub fn lock(&self) -> std::sync::MutexGuard<T> {
        self.inner.lock().expect("Mutex poisoned")
    }
    
    /// Attempts to acquire the lock and returns a guard if successful.
    /// This is a safer alternative to the `lock` method.
    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<T>, &'static str> {
        match self.inner.lock() {
            Ok(guard) => Ok(guard),
            Err(_) => Err("Mutex poisoned"),
        }
    }
}

/// Calculates the sum of two numbers.
/// 
/// # Panics
/// This function will panic if the result overflows.
/// Safety impact: LOW
/// Tracking: WRTQ-500
pub fn add(a: u32, b: u32) -> u32 {
    a.checked_add(b).expect("Addition overflow")
}

/// Safely adds two numbers, returning None if overflow occurs.
/// This is a safer alternative to the `add` function.
pub fn checked_add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bounded_vec() {
        let mut vec = BoundedVec::new(2);
        vec.push(1);
        vec.push(2);
        
        // Would panic:
        // vec.push(3);
        
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), &1);
        assert_eq!(vec.get(1), &2);
        
        // Would panic:
        // vec.get(2);
    }
    
    #[test]
    fn test_try_methods() {
        let mut vec = BoundedVec::new(2);
        assert!(vec.try_push(1).is_ok());
        assert!(vec.try_push(2).is_ok());
        assert!(vec.try_push(3).is_err());
        
        assert_eq!(vec.try_get(0), Some(&1));
        assert_eq!(vec.try_get(1), Some(&2));
        assert_eq!(vec.try_get(2), None);
    }
} 