# `wrt-platform` Safety Documentation

This document outlines the usage of `unsafe` code and FFI within the `wrt-platform` crate,
focusing on aspects relevant to functional safety.

## External Symbols (`extern "C"`)

The `wrt-platform` crate uses Foreign Function Interface (FFI) to interact with platform-specific
APIs for memory allocation and synchronization. These are essential for providing the low-level
abstractions required by the WRT runtime.

All FFI calls are encapsulated within specific modules and wrapped by safe Rust APIs.
Safety invariants for these wrappers are documented in the respective source files.

### 1. `libc` functions (via `macos_memory.rs`)

These functions are used for memory management on macOS when the `platform-macos` feature is enabled.

*   **`mmap(addr: *mut c_void, len: size_t, prot: c_int, flags: c_int, fd: c_int, offset: off_t) -> *mut c_void`**
    *   **Purpose**: Used to allocate virtual memory pages from the operating system.
    *   **Source**: Standard C library (`libc`).
    *   **Usage**: Called within `MacOsAllocator::allocate` to reserve and commit memory.
    *   **Safety**: `mmap` is inherently unsafe as it deals with raw pointers and system resources.
        *   Arguments are carefully constructed (e.g., `MAP_ANON`, `MAP_PRIVATE`, valid protections).
        *   The return value is checked against `MAP_FAILED`.
        *   The lifetime and ownership of the allocated memory are managed by `MacOsAllocator`.

*   **`munmap(addr: *mut c_void, len: size_t) -> c_int`**
    *   **Purpose**: Used to deallocate virtual memory pages.
    *   **Source**: Standard C library (`libc`).
    *   **Usage**: Called within `MacOsAllocator::deallocate` and `MacOsAllocator::drop`.
    *   **Safety**: `munmap` is inherently unsafe.
        *   It must be called with a pointer previously obtained from `mmap` and the correct length of the mapping.
        *   `MacOsAllocator` ensures it uses the correct base pointer and total reserved size.
        *   The caller of `PageAllocator::deallocate` (and `Drop`) must ensure no other references to the memory exist.

*   **`errno()` -> `c_int` (accessed via `libc::errno`)**
    *   **Purpose**: Used to retrieve the system error code after a `libc` call fails.
    *   **Source**: Standard C library (`libc`).
    *   **Usage**: Called in `MacOsAllocator` error paths when `mmap` or `munmap` fail, specifically when the `std` feature is not enabled (otherwise `std::io::Error::last_os_error()` is used).
    *   **Safety**: Reading `errno` is `unsafe` because it's a global mutable variable (thread-local on modern POSIX systems). It must be read immediately after the FFI call that might set it and before any other FFI call that could also modify it. This convention is followed.

### 2. Darwin `_ulock` functions (via `macos_sync.rs`)

These functions are used for futex-like synchronization primitives on macOS when the `platform-macos` feature is enabled. They are part of libSystem.B.dylib on macOS.
See `<sys/ulock.h>` or XNU source for details (e.g., `osfmk/kern/sync_sema.c`).

*   **`_ulock_wait(operation: u32, addr: *mut c_void, value: u64, timeout_us: u32) -> c_int`**
    *   **Purpose**: Atomically checks if the 32-bit integer at `addr` equals `value`, and if so, blocks the calling thread until woken or a timeout occurs.
    *   **Source**: macOS system library.
    *   **Usage**: Called within `MacOsFutex::wait`.
    *   **Safety**: This is an FFI call.
        *   `addr` must point to a valid memory location containing a 32-bit integer. `MacOsFutex` uses the address of an internal `AtomicU32`.
        *   `operation` is fixed to `ULF_WAIT`.
        *   `value` is the expected value, and `timeout_us` is the timeout in microseconds.
        *   Return values are checked for errors like `ETIMEDOUT` or `EINTR`.

*   **`_ulock_wake(operation: u32, addr: *mut c_void, wake_flags: u64) -> c_int`**
    *   **Purpose**: Wakes threads blocked on the futex at `addr` by a previous `_ulock_wait` call.
    *   **Source**: macOS system library.
    *   **Usage**: Called within `MacOsFutex::wake`.
    *   **Safety**: This is an FFI call.
        *   `addr` must point to a valid memory location (the same futex address used in `_ulock_wait`).
        *   `operation` is fixed to `ULF_WAKE`.
        *   `wake_flags` determines whether to wake one or all waiters.
        *   Return values are checked (though `ESRCH` is not treated as a fatal error).

## Unsafe Trait Implementations

*   **`unsafe impl Send for MacOsAllocator`**
*   **`unsafe impl Sync for MacOsAllocator`**
    *   **Justification**: `MacOsAllocator` holds a raw pointer (`base_ptr`) to mmap'd memory. Raw pointers are not `Send` or `Sync`. However, `MacOsAllocator`'s methods that manipulate its state and the memory mapping (`allocate`, `grow`, `deallocate`) take `&mut self`, ensuring exclusive access and preventing data races on the allocator's internal fields. The memory region itself can be shared across threads once allocated, but the safe usage of that shared memory is the responsibility of the memory consumer (e.g., `PalMemoryProvider`). The `MacOsAllocator` itself does not provide methods that would lead to unsafe concurrent access to the memory *through the allocator's own API*. The raw pointer itself can be sent across threads if the `MacOsAllocator` is sent.
    *   Detailed justification is provided in comments within `macos_memory.rs`.

## Unsafe Blocks

Internal `unsafe` blocks are used for:
1.  Calling the FFI functions listed above (`mmap`, `munmap`, `_ulock_wait`, `_ulock_wake`, `errno`).
    *   **Justification**: These are necessary to interact with the operating system.
    *   **Safety Measures**: Each call site is documented with `/// # Safety` comments in the source code, explaining the invariants upheld. Arguments are validated, and return values are checked.

Safety is a primary concern, and all `unsafe` code is localized and aims to provide safe abstractions. 