#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Import appropriate types based on environment
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

use wrt_decoder::instructions::{encode_instruction, parse_instruction};

// Conditional println implementation
#[cfg(feature = "std")]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

#[cfg(not(feature = "std"))]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        // No-op in no_std environment
    };
}

// Standard entry point
#[cfg(feature = "std")]
fn main() {
    test_control_instructions();
}

// No-std entry point
#[cfg(not(feature = "std"))]
fn main() -> ! {
    test_control_instructions();

    // In a real no_std environment, this would be replaced with
    // appropriate platform-specific code
    loop {}
}

fn test_control_instructions() {
    debug_println!("Testing WebAssembly control instructions");

    // Test block instruction
    let block_bytes = vec![0x02, 0x40, 0x0B]; // block (empty) end
    let (block_instr, block_bytes_read) = parse_instruction(&block_bytes).unwrap();
    debug_println!("Parsed block instruction: {:?}", block_instr);
    debug_println!("Bytes read: {}", block_bytes_read);

    let encoded_block = encode_instruction(&block_instr).unwrap();
    debug_println!("Encoded block: {:?}", encoded_block);
    debug_println!(
        "Encoding matches original: {}",
        encoded_block == block_bytes
    );

    // Test loop instruction
    let loop_bytes = vec![
        0x03, 0x7F, // loop with i32 return type
        0x41, 0x01, // i32.const 1
        0x0B, // end
    ];
    let (loop_instr, loop_bytes_read) = parse_instruction(&loop_bytes).unwrap();
    debug_println!("Parsed loop instruction: {:?}", loop_instr);
    debug_println!("Bytes read: {}", loop_bytes_read);

    let encoded_loop = encode_instruction(&loop_instr).unwrap();
    debug_println!("Encoded loop: {:?}", encoded_loop);
    debug_println!("Encoding matches original: {}", encoded_loop == loop_bytes);

    // Test if instruction
    let if_bytes = vec![
        0x04, 0x40, // if with empty block type
        0x41, 0x01, // i32.const 1
        0x05, // else
        0x41, 0x00, // i32.const 0
        0x0B, // end
    ];
    let (if_instr, if_bytes_read) = parse_instruction(&if_bytes).unwrap();
    debug_println!("Parsed if instruction: {:?}", if_instr);
    debug_println!("Bytes read: {}", if_bytes_read);

    let encoded_if = encode_instruction(&if_instr).unwrap();
    debug_println!("Encoded if: {:?}", encoded_if);
    debug_println!("Encoding matches original: {}", encoded_if == if_bytes);

    // Test br_table instruction
    let br_table_bytes = vec![
        0x0E, // br_table
        0x02, // count = 2
        0x00, // label 0
        0x01, // label 1
        0x02, // default label 2
    ];
    let (br_table_instr, br_table_bytes_read) = parse_instruction(&br_table_bytes).unwrap();
    debug_println!("Parsed br_table instruction: {:?}", br_table_instr);
    debug_println!("Bytes read: {}", br_table_bytes_read);

    let encoded_br_table = encode_instruction(&br_table_instr).unwrap();
    debug_println!("Encoded br_table: {:?}", encoded_br_table);
    debug_println!(
        "Encoding matches original: {}",
        encoded_br_table == br_table_bytes
    );

    // Test nested blocks
    let nested_bytes = vec![
        0x02, 0x40, // outer block
        0x02, 0x40, // inner block
        0x0B, // inner end
        0x0B, // outer end
    ];
    let (nested_instr, nested_bytes_read) = parse_instruction(&nested_bytes).unwrap();
    debug_println!("Parsed nested blocks instruction: {:?}", nested_instr);
    debug_println!("Bytes read: {}", nested_bytes_read);

    let encoded_nested = encode_instruction(&nested_instr).unwrap();
    debug_println!("Encoded nested blocks: {:?}", encoded_nested);
    debug_println!(
        "Encoding matches original: {}",
        encoded_nested == nested_bytes
    );
}
