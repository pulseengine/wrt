//! Tests for no_std and no_alloc compatibility

#![no_std]
#![cfg_attr(not(feature = "std"), no_main)]

// Test that we can use the crate without std
use wrt_foundation::prelude::*;

#[cfg(test)]
mod tests {
    use core::mem;

    use super::*;

    #[test]
    fn test_bounded_vec_no_alloc() {
        // Test that BoundedVec works without allocation
        const CAPACITY: usize = 10;
        let provider = NoStdProvider::<{ CAPACITY * 4 }>::default();
        let mut vec: BoundedVec<u32, CAPACITY, NoStdProvider<{ CAPACITY * 4 }>> =
            BoundedVec::new(provider).unwrap();

        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), CAPACITY);

        // Push some values
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0).unwrap(), 1);
        assert_eq!(vec.get(1).unwrap(), 2);
        assert_eq!(vec.get(2).unwrap(), 3);
    }

    #[test]
    fn test_bounded_string_no_alloc() {
        // Test that BoundedString works without allocation
        const CAPACITY: usize = 32;
        let provider = NoStdProvider::<CAPACITY>::default();
        let mut string: BoundedString<CAPACITY, NoStdProvider<CAPACITY>> =
            BoundedString::from_str("", provider).unwrap();

        assert!(string.is_empty());
        assert_eq!(string.len(), 0);

        // Push some characters
        string.push_str("Hello").unwrap();
        assert_eq!(string.as_str().unwrap(), "Hello");

        string.push_str(", World!").unwrap();
        assert_eq!(string.as_str().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_bounded_stack_no_alloc() {
        // Test that BoundedStack works without allocation
        const CAPACITY: usize = 5;
        let provider = NoStdProvider::<{ CAPACITY * 4 }>::default();
        let mut stack: BoundedStack<i32, CAPACITY, NoStdProvider<{ CAPACITY * 4 }>> =
            BoundedStack::new(provider).unwrap();

        assert!(stack.is_empty());

        // Push some values
        stack.push(10).unwrap();
        stack.push(20).unwrap();
        stack.push(30).unwrap();

        assert_eq!(stack.len(), 3);

        // Pop values
        assert_eq!(stack.pop().unwrap(), Some(30));
        assert_eq!(stack.pop().unwrap(), Some(20));
        assert_eq!(stack.pop().unwrap(), Some(10));
        assert_eq!(stack.pop().unwrap(), None);
    }

    #[test]
    fn test_bounded_queue_no_alloc() {
        // Test that BoundedQueue works without allocation
        const CAPACITY: usize = 4;
        let provider = NoStdProvider::<{ CAPACITY * 16 }>::default();
        let mut queue: BoundedQueue<u8, CAPACITY, NoStdProvider<{ CAPACITY * 16 }>> =
            BoundedQueue::new(provider).unwrap();

        assert!(queue.is_empty());

        // Enqueue some values
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        queue.enqueue(3).unwrap();

        assert_eq!(queue.len(), 3);

        // Dequeue values
        assert_eq!(queue.dequeue().unwrap(), Some(1));
        assert_eq!(queue.dequeue().unwrap(), Some(2));
        assert_eq!(queue.dequeue().unwrap(), Some(3));
        assert_eq!(queue.dequeue().unwrap(), None);
    }

    #[test]
    fn test_types_no_alloc() {
        // Test that basic types work without allocation
        let _val_type = ValueType::I32;
        assert_eq!(mem::size_of::<ValueType>(), 1);

        let _ref_type = RefType::Funcref;
        assert_eq!(mem::size_of::<RefType>(), 1);

        // Test limits
        let limits = Limits::new(10, Some(100));
        assert_eq!(limits.min, 10);
        assert_eq!(limits.max, Some(100));
    }

    #[test]
    fn test_verification_no_alloc() {
        // Test verification types work without allocation
        let checksum = Checksum::from_value(0x12345678);
        assert_eq!(checksum.value(), 0x12345678);

        let level = VerificationLevel::Off;
        assert!(matches!(level, VerificationLevel::Off));
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    #[test]
    fn test_simple_hashmap_no_alloc() {
        // Test that SimpleHashMap is available when neither std nor alloc is present
        use wrt_foundation::no_std_hashmap::SimpleHashMap;

        const CAPACITY: usize = 16;
        const PROVIDER_SIZE: usize = CAPACITY * 32; // Enough space for keys and values
        let provider = NoStdProvider::<PROVIDER_SIZE>::default();
        let mut map: SimpleHashMap<u32, u32, CAPACITY, NoStdProvider<PROVIDER_SIZE>> =
            SimpleHashMap::new(provider).unwrap();

        assert!(map.is_empty());

        // Insert some values
        assert!(map.insert(1, 100).unwrap().is_none());
        assert!(map.insert(2, 200).unwrap().is_none());
        assert!(map.insert(3, 300).unwrap().is_none());

        assert_eq!(map.get(&1).unwrap(), Some(100));
        assert_eq!(map.get(&2).unwrap(), Some(200));
        assert_eq!(map.get(&3).unwrap(), Some(300));
        assert_eq!(map.get(&4).unwrap(), None);
    }
}

// Panic handler for no_std environments
#[cfg(all(not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a real embedded system, you might want to log the panic or reset
    loop {}
}

// Entry point for no_std environments
#[cfg(all(not(feature = "std"), not(test)))]
#[no_main]
#[export_name = "_start"]
pub extern "C" fn _start() -> ! {
    // This is just a dummy entry point for no_std compatibility testing
    loop {}
}
