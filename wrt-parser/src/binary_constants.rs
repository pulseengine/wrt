//! WebAssembly binary format constants
//!
//! This module contains all the constants needed for parsing WebAssembly
//! binary format, including magic numbers, section IDs, type encodings,
//! and instruction opcodes.

/// Magic bytes for WebAssembly modules: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// WebAssembly binary format version
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// Magic bytes for WebAssembly Components
pub const COMPONENT_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// WebAssembly Component format version
pub const COMPONENT_VERSION: [u8; 4] = [0x0d, 0x00, 0x01, 0x00];

// Section IDs
pub const CUSTOM_SECTION_ID: u8 = 0x00;
pub const TYPE_SECTION_ID: u8 = 0x01;
pub const IMPORT_SECTION_ID: u8 = 0x02;
pub const FUNCTION_SECTION_ID: u8 = 0x03;
pub const TABLE_SECTION_ID: u8 = 0x04;
pub const MEMORY_SECTION_ID: u8 = 0x05;
pub const GLOBAL_SECTION_ID: u8 = 0x06;
pub const EXPORT_SECTION_ID: u8 = 0x07;
pub const START_SECTION_ID: u8 = 0x08;
pub const ELEMENT_SECTION_ID: u8 = 0x09;
pub const CODE_SECTION_ID: u8 = 0x0A;
pub const DATA_SECTION_ID: u8 = 0x0B;
pub const DATA_COUNT_SECTION_ID: u8 = 0x0C;

// Value types
pub const I32_TYPE: u8 = 0x7F;
pub const I64_TYPE: u8 = 0x7E;
pub const F32_TYPE: u8 = 0x7D;
pub const F64_TYPE: u8 = 0x7C;
pub const V128_TYPE: u8 = 0x7B;
pub const FUNCREF_TYPE: u8 = 0x70;
pub const EXTERNREF_TYPE: u8 = 0x6F;

// Reference types
pub const REF_NULL: u8 = 0x6C;
pub const REF_FUNC: u8 = 0x6B;
pub const REF_EXTERN: u8 = 0x6A;

// Block types
pub const BLOCK_TYPE_EMPTY: u8 = 0x40;

// Control instructions
pub const UNREACHABLE: u8 = 0x00;
pub const NOP: u8 = 0x01;
pub const BLOCK: u8 = 0x02;
pub const LOOP: u8 = 0x03;
pub const IF: u8 = 0x04;
pub const ELSE: u8 = 0x05;
pub const END: u8 = 0x0B;
pub const BR: u8 = 0x0C;
pub const BR_IF: u8 = 0x0D;
pub const BR_TABLE: u8 = 0x0E;
pub const RETURN: u8 = 0x0F;
pub const CALL: u8 = 0x10;
pub const CALL_INDIRECT: u8 = 0x11;

// Parametric instructions
pub const DROP: u8 = 0x1A;
pub const SELECT: u8 = 0x1B;
pub const SELECT_T: u8 = 0x1C;

// Variable instructions
pub const LOCAL_GET: u8 = 0x20;
pub const LOCAL_SET: u8 = 0x21;
pub const LOCAL_TEE: u8 = 0x22;
pub const GLOBAL_GET: u8 = 0x23;
pub const GLOBAL_SET: u8 = 0x24;

// Memory instructions
pub const I32_LOAD: u8 = 0x28;
pub const I64_LOAD: u8 = 0x29;
pub const F32_LOAD: u8 = 0x2A;
pub const F64_LOAD: u8 = 0x2B;
pub const I32_LOAD8_S: u8 = 0x2C;
pub const I32_LOAD8_U: u8 = 0x2D;
pub const I32_LOAD16_S: u8 = 0x2E;
pub const I32_LOAD16_U: u8 = 0x2F;
pub const I64_LOAD8_S: u8 = 0x30;
pub const I64_LOAD8_U: u8 = 0x31;
pub const I64_LOAD16_S: u8 = 0x32;
pub const I64_LOAD16_U: u8 = 0x33;
pub const I64_LOAD32_S: u8 = 0x34;
pub const I64_LOAD32_U: u8 = 0x35;

pub const I32_STORE: u8 = 0x36;
pub const I64_STORE: u8 = 0x37;
pub const F32_STORE: u8 = 0x38;
pub const F64_STORE: u8 = 0x39;
pub const I32_STORE8: u8 = 0x3A;
pub const I32_STORE16: u8 = 0x3B;
pub const I64_STORE8: u8 = 0x3C;
pub const I64_STORE16: u8 = 0x3D;
pub const I64_STORE32: u8 = 0x3E;

pub const MEMORY_SIZE: u8 = 0x3F;
pub const MEMORY_GROW: u8 = 0x40;

// Numeric instructions - constants
pub const I32_CONST: u8 = 0x41;
pub const I64_CONST: u8 = 0x42;
pub const F32_CONST: u8 = 0x43;
pub const F64_CONST: u8 = 0x44;

// Component Model section IDs
pub const COMPONENT_TYPE_SECTION_ID: u8 = 0x01;
pub const COMPONENT_IMPORT_SECTION_ID: u8 = 0x02;
pub const COMPONENT_FUNC_SECTION_ID: u8 = 0x03;
pub const COMPONENT_EXPORT_SECTION_ID: u8 = 0x07;
pub const COMPONENT_START_SECTION_ID: u8 = 0x08;
pub const COMPONENT_INSTANCE_SECTION_ID: u8 = 0x0A;
pub const COMPONENT_ALIAS_SECTION_ID: u8 = 0x0B;