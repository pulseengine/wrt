//! Test no_std compatibility for wrt-format
//!
//! This file validates that the wrt-format crate works correctly in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{format, string::String, vec, vec::Vec};

    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    // Import from wrt-format
    use wrt_format::{
        binary::{
            read_leb128_u32, read_string, write_leb128_u32, write_string, WASM_MAGIC, WASM_VERSION,
        },
        component::ValType,
        module::{Export, ExportKind, Function, Global, Import, ImportDesc, Memory, Table},
        section::{CustomSection, Section, SectionId},
        types::{FormatBlockType, Limits},
    };

    // Import from wrt-types for SafeSlice
    use wrt_types::safe_memory::{SafeSlice, SafeStack};

    #[test]
    fn test_binary_constants() {
        assert_eq!(WASM_MAGIC, [0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(WASM_VERSION, [0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_leb128_encoding() {
        // Test encoding/decoding LEB128
        let mut buffer = vec![0u8; 8];

        // Write u32
        let written = write_leb128_u32(&mut buffer, 624485).unwrap();

        // Read u32
        let (value, read) = read_leb128_u32(&buffer).unwrap();

        // Verify
        assert_eq!(value, 624485);
        assert_eq!(written, read);
    }

    #[test]
    fn test_string_encoding() {
        // Test encoding/decoding strings
        let mut buffer = vec![0u8; 20];
        let test_string = "test_string";

        // Write string
        let written = write_string(&mut buffer, test_string).unwrap();

        // Read string
        let (string, read) = read_string(&buffer).unwrap();

        // Verify
        assert_eq!(string, test_string);
        assert_eq!(written, read);
    }

    #[test]
    fn test_section_ids() {
        // Test section IDs
        assert_eq!(SectionId::Custom.as_u8(), 0);
        assert_eq!(SectionId::Type.as_u8(), 1);
        assert_eq!(SectionId::Import.as_u8(), 2);
        assert_eq!(SectionId::Function.as_u8(), 3);

        // Convert from u8
        assert_eq!(SectionId::from_u8(0), SectionId::Custom);
        assert_eq!(SectionId::from_u8(1), SectionId::Type);
    }

    #[test]
    fn test_custom_section() {
        // Test custom section
        let name = "test_section";
        let data = vec![1, 2, 3, 4];
        let section = CustomSection::new(name.to_string(), data.clone());

        assert_eq!(section.name(), name);
        assert_eq!(section.data(), &data);
    }

    #[test]
    fn test_module_types() {
        // Test export
        let export = Export::new("test_func".to_string(), ExportKind::Function, 0);
        assert_eq!(export.name(), "test_func");
        assert_eq!(export.kind(), ExportKind::Function);
        assert_eq!(export.index(), 0);

        // Test import
        let import_desc = ImportDesc::Function(1);
        let import = Import::new(
            "test_module".to_string(),
            "test_field".to_string(),
            import_desc,
        );
        assert_eq!(import.module(), "test_module");
        assert_eq!(import.name(), "test_field");

        match import.desc() {
            ImportDesc::Function(idx) => assert_eq!(*idx, 1),
            _ => panic!("Wrong import description type"),
        }

        // Test limits
        let limits = Limits::new(1, Some(2), false, false);
        assert_eq!(limits.min, 1);
        assert_eq!(limits.max, Some(2));
        assert_eq!(limits.memory64, false);
        assert_eq!(limits.shared, false);
    }

    #[test]
    fn test_format_val_types() {
        // Test ValType enum
        assert_ne!(ValType::I32, ValType::I64);
        assert_ne!(ValType::F32, ValType::F64);
    }

    #[test]
    fn test_format_block_type() {
        // Test block types
        let block_empty = FormatBlockType::Empty;
        let block_value = FormatBlockType::Value(ValType::I32);

        assert_ne!(block_empty, block_value);
    }

    #[cfg(feature = "safety")]
    #[test]
    fn test_safe_memory_operations() {
        // Test safe memory operations
        use wrt_format::prelude::{memory_provider, safe_slice};

        // Create a sample buffer
        let buffer = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        // Create safe slice
        let safe_buffer = safe_slice(&buffer);

        // Verify first 4 bytes (WASM_MAGIC)
        assert_eq!(&buffer[0..4], &WASM_MAGIC);
        assert_eq!(safe_buffer.range(0, 4).unwrap(), &WASM_MAGIC);

        // Test memory provider
        let provider = memory_provider(buffer.clone());
        let provider_slice = wrt_types::safe_memory::MemoryProvider::borrow_slice(
            &provider,
            0,
            wrt_types::safe_memory::MemoryProvider::size(&provider),
        )
        .unwrap();

        // Verify first 4 bytes (WASM_MAGIC)
        assert_eq!(provider_slice.range(0, 4).unwrap(), &WASM_MAGIC);
    }
}
