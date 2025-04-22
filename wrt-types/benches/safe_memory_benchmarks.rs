//! Performance benchmarks for SafeMemory implementations
//!
//! These benchmarks measure the performance of SafeMemory operations
//! at different verification levels.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use wrt_types::{
    safe_memory::{MemoryProvider, SafeSlice, StdMemoryProvider},
    verification::VerificationLevel,
};

fn safe_memory_store_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("SafeMemory Store");

    // Data to store
    let data = vec![1u8; 1024];

    // Benchmark with verification level None
    {
        let memory = StdMemoryProvider::new(65536);
        let memory_none =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::None);

        group.bench_function("verification_none", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + data.len() <= memory_none.len() {
                        memory_none.copy_from_slice(offset, &data);
                    }
                }
            })
        });
    }

    // Benchmark with verification level Standard
    {
        let memory = StdMemoryProvider::new(65536);
        let memory_standard =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::Standard);

        group.bench_function("verification_standard", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + data.len() <= memory_standard.len() {
                        memory_standard.copy_from_slice(offset, &data);
                    }
                }
            })
        });
    }

    // Benchmark with verification level Full
    {
        let memory = StdMemoryProvider::new(65536);
        let memory_full =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::Full);

        group.bench_function("verification_full", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + data.len() <= memory_full.len() {
                        memory_full.copy_from_slice(offset, &data);
                    }
                }
            })
        });
    }

    group.finish();
}

fn safe_memory_load_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("SafeMemory Load");

    // Setup memory with data
    let memory = StdMemoryProvider::new(65536);
    let setup_slice = SafeSlice::new(memory.get_buffer());
    for i in 0..65536 {
        setup_slice.set(i, (i % 256) as u8);
    }

    // Benchmark with verification level None
    {
        let memory_none =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::None);

        group.bench_function("verification_none", |b| {
            b.iter(|| {
                let mut sum = 0u8;
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + 1024 <= memory_none.len() {
                        for j in 0..1024 {
                            sum = sum.wrapping_add(memory_none.get(offset + j));
                        }
                    }
                }
                black_box(sum);
            })
        });
    }

    // Benchmark with verification level Standard
    {
        let memory_standard =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::Standard);

        group.bench_function("verification_standard", |b| {
            b.iter(|| {
                let mut sum = 0u8;
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + 1024 <= memory_standard.len() {
                        for j in 0..1024 {
                            sum = sum.wrapping_add(memory_standard.get(offset + j));
                        }
                    }
                }
                black_box(sum);
            })
        });
    }

    // Benchmark with verification level Full
    {
        let memory_full =
            SafeSlice::with_verification_level(memory.get_buffer(), VerificationLevel::Full);

        group.bench_function("verification_full", |b| {
            b.iter(|| {
                let mut sum = 0u8;
                for i in 0..10 {
                    let offset = i * 1024;
                    if offset + 1024 <= memory_full.len() {
                        for j in 0..1024 {
                            sum = sum.wrapping_add(memory_full.get(offset + j));
                        }
                    }
                }
                black_box(sum);
            })
        });
    }

    group.finish();
}

fn memory_adapter_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Adapter");

    // Setup data
    let data = vec![1u8; 1024];

    // Create memory with different adapters
    let memory = Arc::new(wrt_runtime::Memory::new(1).unwrap());

    // Default adapter benchmark
    {
        let adapter = wrt::memory_adapter::DefaultMemoryAdapter::new(memory.clone());

        group.bench_function("default_adapter", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    let _ = adapter.store(offset, &data);
                    let _ = adapter.load(offset, 1024);
                }
            })
        });
    }

    // Safe adapter with no verification
    {
        let adapter = wrt::memory_adapter::SafeMemoryAdapter::with_verification_level(
            memory.clone(),
            VerificationLevel::None,
        );

        group.bench_function("safe_adapter_none", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    let _ = adapter.store(offset, &data);
                    let _ = adapter.load(offset, 1024);
                }
            })
        });
    }

    // Safe adapter with full verification
    {
        let adapter = wrt::memory_adapter::SafeMemoryAdapter::with_verification_level(
            memory.clone(),
            VerificationLevel::Full,
        );

        group.bench_function("safe_adapter_full", |b| {
            b.iter(|| {
                for i in 0..10 {
                    let offset = i * 1024;
                    let _ = adapter.store(offset, &data);
                    let _ = adapter.load(offset, 1024);
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    safe_memory_store_benchmark,
    safe_memory_load_benchmark,
    memory_adapter_benchmark
);
criterion_main!(benches);
