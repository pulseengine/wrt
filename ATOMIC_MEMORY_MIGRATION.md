# Atomic Memory Operations Migration Guide

## Background

We've identified a critical vulnerability in the current WebAssembly Runtime (WRT) memory safety implementation: bit flips can occur in the time window between writing data to memory and calculating/updating its checksum. This vulnerability can lead to undetected memory corruption, as the checksum doesn't accurately reflect the actual data state.

To address this issue, we've implemented a new `AtomicMemoryOps` component that ensures atomic operations for memory writes and checksum calculations using mutex-based synchronization.

## Operations That Should Be Updated

The following types of operations should be migrated to use the new `AtomicMemoryOps` implementation:

1. Direct calls to `SliceMut::data_mut()` followed by `SliceMut::update_checksum()`
2. Memory write operations in `SafeMemoryHandler`
3. Any critical data integrity verification operations in:
   - `BoundedVec`
   - `BoundedStack`
   - `Memory` operations (especially `Memory::write` and `Memory::copy`)
   - Resource management operations
   - Component data serialization/deserialization

## Migration Steps

### 1. Update Import Statements

Add the following import to your module:

```rust
use wrt_foundation::prelude::{AtomicMemoryOps, AtomicMemoryExt};
```

### 2. For Existing SafeMemoryHandler Users:

Convert your existing `SafeMemoryHandler<P>` to an `AtomicMemoryOps<P>`:

```rust
// Before
let handler = SafeMemoryHandler::new(provider);

// After
let atomic_ops = handler.into_atomic_ops().unwrap();
// OR directly from provider
let atomic_ops = provider.into_atomic_ops().unwrap();
```

### 3. Replace Write + Checksum Patterns

Replace sequences of write + checksum operations with atomic alternatives:

```rust
// Before
let mut slice = handler.get_slice_mut(offset, len)?;
let slice_data = slice.data_mut()?;
slice_data.copy_from_slice(data);
slice.update_checksum();

// After
atomic_ops.atomic_write_with_checksum(offset, data)?;
```

### 4. Replace Copy Operations

Update copy operations to use atomic alternatives:

```rust
// Before
handler.copy_within(src_offset, dst_offset, len)?;

// After
atomic_ops.atomic_copy_within(src_offset, dst_offset, len)?;
```

### 5. For Memory Implementation

The `Memory` struct should adopt atomic operations for critical write operations:

```rust
// Before
self.data.write_data(addr, data)?;
if self.verification_level == VerificationLevel::Full {
    self.data.verify_integrity()?;
}

// After
let atomic_ops = self.data.into_atomic_ops()?;
atomic_ops.atomic_write_with_checksum(addr, data)?;
```

## VerificationLevel Compatibility

The `AtomicMemoryOps` implementation respects the existing `VerificationLevel` settings:

- `VerificationLevel::Off`: No checksumming performed
- `VerificationLevel::Sampling`: Checksums calculated probabilistically
- `VerificationLevel::Standard`: Checksums calculated at key operations
- `VerificationLevel::Full`: Checksums always calculated and verified

## Thread Safety Considerations

- `AtomicMemoryOps` is thread-safe for both read and write operations
- The internal mutex ensures that only one thread can perform write operations at a time
- Multiple concurrent reads are still allowed when using borrow_slice
- The implementation minimizes lock contention by releasing locks as soon as operations complete

## Performance Implications

Using `AtomicMemoryOps` adds a small overhead due to mutex synchronization, but this is necessary to ensure data integrity. The performance impact should be minimal for most use cases, especially compared to the safety benefits.

## No_std Compatibility

The `AtomicMemoryOps` implementation is fully compatible with no_std environments:
- Uses core::sync::atomic and wrt_sync::mutex for synchronization
- Does not require heap allocations
- Maintains compact memory footprint

## Testing Your Migration

After migrating to `AtomicMemoryOps`, verify that:

1. All critical memory operations with checksumming use atomic operations
2. Data integrity verification still passes
3. Unit tests continue to pass
4. Performance remains acceptable

## Example Implementation

```rust
use wrt_foundation::prelude::{AtomicMemoryOps, AtomicMemoryExt, NoStdProvider};

// Create a provider
let provider = NoStdProvider::<1024>::new();

// Create atomic memory operations handler
let atomic_ops = provider.into_atomic_ops().unwrap();

// Write data with atomic checksumming
let data = [1, 2, 3, 4, 5];
atomic_ops.atomic_write_with_checksum(0, &data).unwrap();

// Read data back
let slice = atomic_ops.borrow_slice(0, data.len()).unwrap();
let read_data = slice.data().unwrap();

assert_eq!(read_data, &data);
```