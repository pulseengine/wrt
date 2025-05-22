// Test to verify compatibility between std and no_std modes
// This file should work with both feature sets

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use wrt_decoder::parser::Parser;
use wrt_format::binary::{WASM_MAGIC, WASM_VERSION};
use wrt_foundation::{
    safe_memory::{MemoryProvider, SafeSlice, StdMemoryProvider},
    verification::VerificationLevel,
};

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_wasm_header_parsing() {
    // Create a minimal valid WebAssembly module with just the header
    let mut module = Vec::new();
    module.extend_from_slice(&WASM_MAGIC);
    module.extend_from_slice(&WASM_VERSION);

    // Parse with direct slice
    let mut parser = Parser::new(Some(&module[..]), false);
    let version_payload = parser.next().unwrap().unwrap();

    // Check that we got version 1
    match version_payload {
        wrt_decoder::parser::Payload::Version(1, _) => {
            // This is correct
        }
        other => panic!("Unexpected payload: {:?}", other),
    }

    // Parse with SafeSlice for memory safety
    let safe_slice = SafeSlice::new(&module);
    let mut safe_parser = Parser::from_safe_slice(safe_slice);
    let safe_version_payload = safe_parser.next().unwrap().unwrap();

    // Check that we got the same result
    match safe_version_payload {
        wrt_decoder::parser::Payload::Version(1, _) => {
            // This is correct
        }
        other => panic!("Unexpected payload: {:?}", other),
    }

    // Use memory provider
    let memory_provider = StdMemoryProvider::new(module);
    let provider_slice =
        MemoryProvider::borrow_slice(&memory_provider, 0, MemoryProvider::size(&memory_provider))
            .unwrap();
    let mut provider_parser = Parser::from_safe_slice(provider_slice);
    let provider_version_payload = provider_parser.next().unwrap().unwrap();

    // Check that we got the same result again
    match provider_version_payload {
        wrt_decoder::parser::Payload::Version(1, _) => {
            // This is correct
        }
        other => panic!("Unexpected payload: {:?}", other),
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_verification_levels() {
    // Create a minimal valid WebAssembly module with just the header
    let mut module = Vec::new();
    module.extend_from_slice(&WASM_MAGIC);
    module.extend_from_slice(&WASM_VERSION);

    // Test with different verification levels
    let none_slice = SafeSlice::with_verification_level(&module, VerificationLevel::None);
    let sampling_slice = SafeSlice::with_verification_level(&module, VerificationLevel::Sampling);
    let standard_slice = SafeSlice::with_verification_level(&module, VerificationLevel::Standard);
    let full_slice = SafeSlice::with_verification_level(&module, VerificationLevel::Full);

    // Verify they all work
    Parser::from_safe_slice(none_slice).next().unwrap().unwrap();
    Parser::from_safe_slice(sampling_slice).next().unwrap().unwrap();
    Parser::from_safe_slice(standard_slice).next().unwrap().unwrap();
    Parser::from_safe_slice(full_slice).next().unwrap().unwrap();
}

// Add a simple test that runs in pure no_std mode (no alloc)
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[test]
fn test_pure_nostd_header_validation() {
    // In pure no_std mode, we can still use SafeSlice with static data
    let wasm_header = [
        // WASM_MAGIC
        0x00, 0x61, 0x73, 0x6d, // WASM_VERSION
        0x01, 0x00, 0x00, 0x00,
    ];

    // We can validate the header
    let safe_slice = SafeSlice::new(&wasm_header);
    let mut parser = Parser::from_safe_slice(safe_slice);

    // Check the version payload
    let version_payload = parser.next().unwrap().unwrap();
    match version_payload {
        wrt_decoder::parser::Payload::Version(1, _) => {
            // This is correct
        }
        other => panic!("Unexpected payload: {:?}", other),
    }
}
