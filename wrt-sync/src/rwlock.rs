use core::{
    cell::UnsafeCell,
    fmt,
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(feature = "std")]
use std::{sync::atomic::AtomicBool, sync::Arc, thread, vec::Vec};

/// A simple, `no_std` compatible Read-Write Lock using atomics.
///
/// This is a reader-preference lock. Writers may starve if readers constantly hold the lock.
/// WARNING: This is a basic implementation. It does not handle potential deadlocks involving multiple locks
/// or priority inversion. Use with caution.
const WRITE_LOCK_STATE: usize = usize::MAX;

pub struct WrtRwLock<T: ?Sized> {
    /// Atomically tracks the lock state.
    /// Encoding:
    /// - 0: Unlocked
    /// - usize::MAX: Write-locked
    /// - n (1..usize::MAX-1): Read-locked by n readers
    state: AtomicUsize,
    data: UnsafeCell<T>,
}

/// A guard that provides read access to the data protected by a `WrtRwLock`.
#[clippy::has_significant_drop]
pub struct WrtRwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a WrtRwLock<T>,
}

/// A guard that provides write access to the data protected by a `WrtRwLock`.
#[clippy::has_significant_drop]
pub struct WrtRwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a WrtRwLock<T>,
}

// Allow the lock to be shared across threads.
// Safety: Requires correct implementation of locking mechanisms.
unsafe impl<T: ?Sized + Send + Sync> Send for WrtRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for WrtRwLock<T> {}

impl<T> WrtRwLock<T> {
    /// Creates a new `WrtRwLock` protecting the given data.
    #[inline]
    pub const fn new(data: T) -> Self {
        WrtRwLock {
            state: AtomicUsize::new(0), // Start unlocked
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> WrtRwLock<T> {
    /// Acquires a read lock, spinning until available.
    #[inline]
    pub fn read(&self) -> WrtRwLockReadGuard<'_, T> {
        loop {
            let current_state = self.state.load(Ordering::Relaxed);
            // Check if write-locked
            if current_state != WRITE_LOCK_STATE {
                // Attempt to increment reader count
                match self.state.compare_exchange_weak(
                    current_state,
                    current_state + 1,
                    Ordering::Acquire, // Ensure reads happen after lock acquisition
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return WrtRwLockReadGuard { lock: self },
                    Err(_) => continue, // State changed, retry loop
                }
            }
            // If write-locked, spin
            spin_loop();
        }
    }

    /// Acquires a write lock, spinning until available.
    #[inline]
    pub fn write(&self) -> WrtRwLockWriteGuard<'_, T> {
        loop {
            // Attempt to acquire write lock if currently unlocked (state == 0)
            match self.state.compare_exchange_weak(
                0, // Only try if unlocked
                WRITE_LOCK_STATE,
                Ordering::Acquire, // Ensure writes happen after lock acquisition
                Ordering::Relaxed,
            ) {
                Ok(_) => return WrtRwLockWriteGuard { lock: self },
                Err(current_state) => {
                    // If it failed because it was already locked (read or write), spin.
                    // If current_state was 0 but compare_exchange failed spuriously,
                    // the loop retries anyway.
                    if current_state != 0 {
                        spin_loop();
                    }
                    // Continue loop regardless to retry compare_exchange
                }
            }
        }
    }

    // Optional: Implement try_read, try_write (left as exercise)
    /*
    #[inline]
    pub fn try_read(&self) -> Option<WrtRwLockReadGuard<'_, T>> {
        let current_state = self.state.load(Ordering::Relaxed);
        if current_state != WRITE_LOCK_STATE {
            match self.state.compare_exchange( // Use strong exchange for try_ versions
                current_state,
                current_state + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => Some(WrtRwLockReadGuard { lock: self }),
                Err(_) => None, // Failed to acquire (state changed)
            }
        } else {
            None // Write locked
        }
    }

    #[inline]
    pub fn try_write(&self) -> Option<WrtRwLockWriteGuard<'_, T>> {
        match self.state.compare_exchange( // Use strong exchange for try_ versions
            0,
            WRITE_LOCK_STATE,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Some(WrtRwLockWriteGuard { lock: self }),
            Err(_) => None, // Failed (already locked or state changed)
        }
    }
    */
}

// Read Guard Implementation

impl<T: ?Sized> Deref for WrtRwLockReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        // Safety: Guard ensures read lock is held.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for WrtRwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Decrement reader count.
        // Release ordering ensures preceding reads are visible before the lock is released (partially).
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

// Write Guard Implementation

impl<T: ?Sized> Deref for WrtRwLockWriteGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        // Safety: Guard ensures write lock is held.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for WrtRwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: Guard ensures write lock is held.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for WrtRwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Set state back to unlocked (0).
        // Release ordering ensures preceding writes are visible before the lock is released.
        self.lock.state.store(0, Ordering::Release);
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for WrtRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Attempt a non-blocking read for Debug representation if possible,
        // otherwise indicate locked status. Avoids deadlocking Debug.
        let current_state = self.state.load(Ordering::Relaxed);
        if current_state == 0 {
            // Briefly try to show unlocked state data. Imperfect.
            // A try_read would be safer if available and implemented.
            f.debug_struct("WrtRwLock")
                .field("state", &"Unlocked")
                .field("data", unsafe { &&*self.data.get() }) // Unsafe access for debug only
                .finish()
        } else if current_state == WRITE_LOCK_STATE {
            f.debug_struct("WrtRwLock")
                .field("state", &"WriteLocked")
                .field("data", &"<locked>")
                .finish()
        } else {
            f.debug_struct("WrtRwLock")
                .field("state", &format_args!("ReadLocked({})", current_state))
                .field("data", &"<locked>") // Avoid showing data during read lock in Debug
                .finish()
        }
    }
}

// ======= Parking-based Implementation (for std environments) =======
#[cfg(feature = "std")]
pub struct WrtParkingRwLock<T: ?Sized> {
    /// Atomically tracks the lock state.
    /// Encoding:
    /// - 0: Unlocked
    /// - usize::MAX: Write-locked
    /// - n (1..usize::MAX-1): Read-locked by n readers
    state: AtomicUsize,
    /// Flag to indicate a waiting writer
    writer_waiting: AtomicBool,
    /// Queue of waiting readers/writers to be woken up
    waiters: Arc<WaitQueue>,
    /// Protected data
    data: UnsafeCell<T>,
}

#[cfg(feature = "std")]
struct WaitQueue {
    // Track reader threads in a vector behind RwLock for interior mutability
    readers: std::sync::RwLock<Vec<thread::Thread>>,
    // Writer thread (only one can wait at a time)
    writer: std::sync::RwLock<Option<thread::Thread>>,
    readers_count: AtomicUsize,
    writer_waiting: AtomicBool,
}

#[cfg(feature = "std")]
impl WaitQueue {
    fn new() -> Self {
        Self {
            readers: std::sync::RwLock::new(Vec::new()),
            writer: std::sync::RwLock::new(None),
            readers_count: AtomicUsize::new(0),
            writer_waiting: AtomicBool::new(false),
        }
    }

    fn register_reader(&self) {
        let current = thread::current();
        // Add to reader queue
        if let Ok(mut readers) = self.readers.write() {
            readers.push(current);
        }
        self.readers_count.fetch_add(1, Ordering::Relaxed);
    }

    fn unregister_reader(&self) {
        // This is simple bookkeeping, the reader already acquired the lock
        self.readers_count.fetch_sub(1, Ordering::Relaxed);
    }

    fn register_writer(&self) -> bool {
        let current = thread::current();
        // Only one writer can wait at a time
        if self.writer_waiting.swap(true, Ordering::Relaxed) {
            // Another writer is already waiting
            return false;
        }

        // Set the writer thread
        if let Ok(mut writer) = self.writer.write() {
            *writer = Some(current);
        }
        true
    }

    fn unregister_writer(&self) {
        self.writer_waiting.store(false, Ordering::Relaxed);
        // Clear the writer thread
        if let Ok(mut writer) = self.writer.write() {
            *writer = None;
        }
    }

    // Wake one writer if there's any waiting
    fn wake_writer(&self) -> bool {
        if let Ok(writer) = self.writer.read() {
            if let Some(thread) = writer.as_ref() {
                thread.unpark();
                return true;
            }
        }
        false
    }

    // Wake all waiting readers
    fn wake_readers(&self) {
        if let Ok(readers) = self.readers.read() {
            for thread in readers.iter() {
                thread.unpark();
            }
        }

        // Clear readers list after unparking
        if let Ok(mut readers) = self.readers.write() {
            readers.clear();
        }
    }
}

#[cfg(feature = "std")]
unsafe impl<T: ?Sized + Send + Sync> Send for WrtParkingRwLock<T> {}

#[cfg(feature = "std")]
unsafe impl<T: ?Sized + Send + Sync> Sync for WrtParkingRwLock<T> {}

#[cfg(feature = "std")]
#[clippy::has_significant_drop]
pub struct WrtParkingRwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a WrtParkingRwLock<T>,
}

#[cfg(feature = "std")]
#[clippy::has_significant_drop]
pub struct WrtParkingRwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a WrtParkingRwLock<T>,
}

#[cfg(feature = "std")]
impl<T> WrtParkingRwLock<T> {
    /// Creates a new `WrtParkingRwLock` protecting the given data.
    #[inline]
    pub fn new(data: T) -> Self {
        WrtParkingRwLock {
            state: AtomicUsize::new(0), // Start unlocked
            writer_waiting: AtomicBool::new(false),
            waiters: Arc::new(WaitQueue::new()),
            data: UnsafeCell::new(data),
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> WrtParkingRwLock<T> {
    /// Acquires a read lock, parking the thread if not immediately available.
    #[inline]
    pub fn read(&self) -> WrtParkingRwLockReadGuard<'_, T> {
        let waiters = Arc::clone(&self.waiters);
        let mut registered = false;

        loop {
            let current_state = self.state.load(Ordering::Relaxed);

            // Check if write-locked
            if current_state != WRITE_LOCK_STATE && !self.writer_waiting.load(Ordering::Relaxed) {
                // Attempt to increment reader count
                match self.state.compare_exchange_weak(
                    current_state,
                    current_state + 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Successfully acquired read lock
                        if registered {
                            waiters.unregister_reader();
                        }
                        return WrtParkingRwLockReadGuard { lock: self };
                    }
                    Err(_) => {
                        // Failed to acquire lock, continue to register and park
                    }
                }
            }

            // Register as waiting if not already registered
            if !registered {
                waiters.register_reader();
                registered = true;
            }

            // Park the thread
            thread::park();
        }
    }

    /// Acquires a write lock, parking the thread if not immediately available.
    #[inline]
    pub fn write(&self) -> WrtParkingRwLockWriteGuard<'_, T> {
        let waiters = Arc::clone(&self.waiters);
        let mut registered = false;

        // Signal that a writer is waiting
        self.writer_waiting.store(true, Ordering::Relaxed);

        loop {
            // Try to acquire the write lock if unlocked
            match self.state.compare_exchange_weak(
                0,
                WRITE_LOCK_STATE,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Successfully acquired write lock
                    self.writer_waiting.store(false, Ordering::Relaxed);
                    if registered {
                        waiters.unregister_writer();
                    }
                    return WrtParkingRwLockWriteGuard { lock: self };
                }
                Err(_) => {
                    // Failed to acquire lock, register and park
                    if !registered && waiters.register_writer() {
                        registered = true;
                    }

                    // Park the thread
                    thread::park();
                }
            }
        }
    }

    /// Try to acquire a read lock without blocking.
    #[inline]
    pub fn try_read(&self) -> Option<WrtParkingRwLockReadGuard<'_, T>> {
        let current_state = self.state.load(Ordering::Relaxed);

        // Can't acquire if write locked or writer waiting
        if current_state == WRITE_LOCK_STATE || self.writer_waiting.load(Ordering::Relaxed) {
            return None;
        }

        // Try to increment reader count
        match self.state.compare_exchange(
            current_state,
            current_state + 1,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Some(WrtParkingRwLockReadGuard { lock: self }),
            Err(_) => None,
        }
    }

    /// Try to acquire a write lock without blocking.
    #[inline]
    pub fn try_write(&self) -> Option<WrtParkingRwLockWriteGuard<'_, T>> {
        // Try to acquire the write lock if unlocked
        match self
            .state
            .compare_exchange(0, WRITE_LOCK_STATE, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => Some(WrtParkingRwLockWriteGuard { lock: self }),
            Err(_) => None,
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> Drop for WrtParkingRwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Decrement reader count
        let old_state = self.lock.state.fetch_sub(1, Ordering::Release);

        // If this was the last reader and a writer is waiting, wake up a writer
        if old_state == 1 && self.lock.writer_waiting.load(Ordering::Relaxed) {
            // Unpark a waiting writer thread
            let waiters = Arc::clone(&self.lock.waiters);
            if waiters.writer_waiting.load(Ordering::Relaxed) {
                // Wake the writer
                waiters.wake_writer();
            }
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> Drop for WrtParkingRwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Set state back to unlocked
        self.lock.state.store(0, Ordering::Release);

        // Wake up waiting readers or writers
        let waiters = Arc::clone(&self.lock.waiters);
        let readers_count = waiters.readers_count.load(Ordering::Relaxed);
        let writer_waiting = waiters.writer_waiting.load(Ordering::Relaxed);

        // To maintain writer fairness, prefer waking a writer if one is waiting
        if writer_waiting {
            waiters.wake_writer();
        } else if readers_count > 0 {
            // If no writers, wake all readers
            waiters.wake_readers();
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for WrtParkingRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let current_state = self.state.load(Ordering::Relaxed);
        if current_state == 0 {
            f.debug_struct("WrtParkingRwLock")
                .field("state", &"Unlocked")
                .field("data", unsafe { &&*self.data.get() })
                .finish()
        } else if current_state == WRITE_LOCK_STATE {
            f.debug_struct("WrtParkingRwLock")
                .field("state", &"WriteLocked")
                .field("data", &"<locked>")
                .finish()
        } else {
            f.debug_struct("WrtParkingRwLock")
                .field("state", &format_args!("ReadLocked({})", current_state))
                .field("data", &"<locked>")
                .finish()
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> Deref for WrtParkingRwLockReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        // Safety: Guard ensures read lock is held.
        unsafe { &*self.lock.data.get() }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> Deref for WrtParkingRwLockWriteGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        // Safety: Guard ensures write lock is held.
        unsafe { &*self.lock.data.get() }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> DerefMut for WrtParkingRwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: Guard ensures write lock is held.
        unsafe { &mut *self.lock.data.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use alloc types for tests when not using std
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    // Use std types when std feature is enabled
    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    #[test]
    fn test_rwlock_new_read() {
        let lock = WrtRwLock::new(42);
        let data = lock.read();
        assert_eq!(*data, 42);
    }

    #[test]
    fn test_rwlock_new_write_read() {
        let lock = WrtRwLock::new(42);
        {
            let mut writer = lock.write();
            *writer = 100;
        }
        let data = lock.read();
        assert_eq!(*data, 100);
    }

    #[test]
    fn test_rwlock_multiple_readers() {
        let lock = WrtRwLock::new(String::from("hello"));
        let r1 = lock.read();
        let r2 = lock.read();
        assert_eq!(*r1, "hello");
        assert_eq!(*r2, "hello");
        // Drop guards explicitly to show order (though not strictly necessary)
        drop(r1);
        drop(r2);
    }

    #[test]
    fn test_rwlock_write_blocks_write() {
        // This test logic requires threads to actually test blocking.
        // We simulate the state change.
        let lock = WrtRwLock::new(0);
        let _w1 = lock.write(); // Acquire write lock
                                // Try to acquire another write lock (conceptually) - it should block.
                                // In a single thread, lock.write() would just spin infinitely here.
                                // We can check the state:
        assert_eq!(lock.state.load(Ordering::Relaxed), WRITE_LOCK_STATE);
    }

    #[test]
    fn test_rwlock_read_blocks_write() {
        let lock = WrtRwLock::new(0);
        let _r1 = lock.read(); // Acquire read lock
                               // Try to acquire write lock (conceptually) - it should block.
                               // In a single thread, lock.write() would spin infinitely.
                               // Check the state:
        assert_eq!(lock.state.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_rwlock_write_blocks_read() {
        let lock = WrtRwLock::new(0);
        let _w1 = lock.write(); // Acquire write lock
                                // Try to acquire read lock (conceptually) - it should block.
                                // In a single thread, lock.read() would spin infinitely.
                                // Check the state:
        assert_eq!(lock.state.load(Ordering::Relaxed), WRITE_LOCK_STATE);
    }

    // Basic concurrency tests (require std for threading)
    #[cfg(feature = "std")]
    mod concurrency {
        use super::*; // Access WrtRwLock etc.
        use std::println;
        use std::sync::{Arc, Barrier};
        use std::thread; // Import println for the tests

        #[test]
        fn test_rwlock_concurrent_reads() {
            let lock = Arc::new(WrtRwLock::new(123));
            let num_threads = 10;
            let barrier = Arc::new(Barrier::new(num_threads));
            let mut handles = vec![];

            for _ in 0..num_threads {
                let lock_clone = Arc::clone(&lock);
                let barrier_clone = Arc::clone(&barrier);
                handles.push(thread::spawn(move || {
                    barrier_clone.wait(); // Sync threads
                    let reader = lock_clone.read();
                    assert_eq!(*reader, 123);
                    // Keep lock for a bit
                    thread::sleep(std::time::Duration::from_millis(10));
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }
            assert_eq!(lock.state.load(Ordering::Relaxed), 0); // Should be unlocked now
        }

        #[test]
        fn test_rwlock_concurrent_write_then_reads() {
            let lock = Arc::new(WrtRwLock::new(0));
            let num_readers = 5;
            let barrier = Arc::new(Barrier::new(num_readers + 1)); // +1 for writer
            let reader_start = Arc::new(Barrier::new(num_readers + 1)); // Wait for writer to finish
            let mut handles = vec![];

            // Writer thread
            let lock_clone_w = Arc::clone(&lock);
            let barrier_clone_w = Arc::clone(&barrier);
            let reader_start_clone = Arc::clone(&reader_start);
            handles.push(thread::spawn(move || {
                barrier_clone_w.wait();
                let mut writer = lock_clone_w.write();
                *writer = 999;
                println!("Writer finished setting value to 999");
                // Hold lock for a while
                thread::sleep(std::time::Duration::from_millis(50));
                // Explicitly drop to ensure release before readers start
                drop(writer);
                // Signal readers to start
                reader_start_clone.wait();
            }));

            // Reader threads
            for i in 0..num_readers {
                let lock_clone_r = Arc::clone(&lock);
                let barrier_clone_r = Arc::clone(&barrier);
                let reader_start_clone = Arc::clone(&reader_start);
                handles.push(thread::spawn(move || {
                    barrier_clone_r.wait(); // Meet with writer
                                            // Wait for writer to finish
                    reader_start_clone.wait();
                    println!("Reader {} starting", i);
                    // Readers should see writer's change
                    let reader = lock_clone_r.read();
                    println!("Reader {} got value: {}", i, *reader);
                    assert_eq!(*reader, 999);
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
            assert_eq!(lock.state.load(Ordering::Relaxed), 0);
        }

        #[test]
        fn test_rwlock_concurrent_reads_then_write() {
            let lock = Arc::new(WrtRwLock::new(String::from("initial")));
            let num_readers = 5;
            let reader_barrier = Arc::new(Barrier::new(num_readers));
            let main_barrier = Arc::new(Barrier::new(2)); // Sync reader group and writer
            let mut reader_handles = vec![];

            // Reader threads
            for i in 0..num_readers {
                let lock_clone_r = Arc::clone(&lock);
                let reader_barrier_clone = Arc::clone(&reader_barrier);
                let main_barrier_clone = Arc::clone(&main_barrier);
                reader_handles.push(thread::spawn(move || {
                    println!("Reader {} acquiring", i);
                    let reader = lock_clone_r.read();
                    assert_eq!(*reader, "initial");
                    println!("Reader {} acquired, waiting at barrier", i);
                    reader_barrier_clone.wait(); // Wait for all readers to acquire
                    println!("Reader {} passed barrier, waiting for main barrier", i);
                    if i == 0 {
                        // Let one reader sync with main
                        main_barrier_clone.wait();
                    }
                    println!("Reader {} releasing", i);
                    // Keep lock until main barrier is passed
                })); // Read locks released here
            }

            // Writer thread - should block until all readers release
            let lock_clone_w = Arc::clone(&lock);
            let main_barrier_clone_w = Arc::clone(&main_barrier);
            let writer_handle = thread::spawn(move || {
                println!("Writer waiting for main barrier");
                main_barrier_clone_w.wait(); // Wait until readers are holding locks
                println!("Writer trying to acquire");
                let mut writer = lock_clone_w.write(); // This should block
                println!("Writer acquired");
                *writer = String::from("modified");
            });

            // Wait for readers first, then writer
            for handle in reader_handles {
                handle.join().unwrap();
            }
            writer_handle.join().unwrap();

            // Verify final state
            let final_reader = lock.read();
            assert_eq!(*final_reader, "modified");
            assert_eq!(lock.state.load(Ordering::Relaxed), 1); // Final reader holds lock
        }
    }

    // Tests for WrtParkingRwLock
    #[cfg(feature = "std")]
    mod parking_tests {
        use super::*;
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_parking_rwlock_basic() {
            let lock = WrtParkingRwLock::new(42);
            let reader = lock.read();
            assert_eq!(*reader, 42);
            drop(reader);

            let mut writer = lock.write();
            *writer = 100;
            drop(writer);

            let reader = lock.read();
            assert_eq!(*reader, 100);
        }

        #[test]
        fn test_parking_rwlock_concurrent_reads() {
            let lock = Arc::new(WrtParkingRwLock::new(123));
            let mut handles = vec![];

            for _ in 0..5 {
                let lock_clone = Arc::clone(&lock);
                handles.push(thread::spawn(move || {
                    let reader = lock_clone.read();
                    thread::sleep(Duration::from_millis(10));
                    assert_eq!(*reader, 123);
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }

        #[test]
        fn test_parking_rwlock_writer_blocks_readers() {
            let lock = Arc::new(WrtParkingRwLock::new(0));
            let lock_clone_w = Arc::clone(&lock);

            // Start writer thread that holds lock for a while
            let writer_handle = thread::spawn(move || {
                let mut writer = lock_clone_w.write();
                *writer = 999;
                thread::sleep(Duration::from_millis(50));
                drop(writer);
            });

            // Give writer time to acquire lock
            thread::sleep(Duration::from_millis(10));

            // Start reader thread, should block until writer releases
            let lock_clone_r = Arc::clone(&lock);
            let reader_handle = thread::spawn(move || {
                let reader = lock_clone_r.read();
                assert_eq!(*reader, 999); // Should see writer's change
            });

            writer_handle.join().unwrap();
            reader_handle.join().unwrap();
        }

        #[test]
        fn test_parking_rwlock_try_operations() {
            let lock = WrtParkingRwLock::new(42);

            // Try operations when unlocked
            assert!(lock.try_read().is_some());

            // First make sure no readers are active
            {
                let reader = lock.try_read();
                assert!(reader.is_some());
                // Drop explicitly to ensure lock is released
                drop(reader.unwrap());
            }

            // Now try write
            let writer = lock.try_write();
            assert!(writer.is_some());

            // Can't read while write locked
            assert!(lock.try_read().is_none());
        }
    }
}
