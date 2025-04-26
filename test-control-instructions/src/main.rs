use wrt_decoder::instructions::{encode_instruction, parse_instruction};

fn main() {
    println!("Testing WebAssembly control instructions");

    // Test block instruction
    let block_bytes = vec![0x02, 0x40, 0x0B]; // block (empty) end
    let (block_instr, block_bytes_read) = parse_instruction(&block_bytes).unwrap();
    println!("Parsed block instruction: {:?}", block_instr);
    println!("Bytes read: {}", block_bytes_read);

    let encoded_block = encode_instruction(&block_instr).unwrap();
    println!("Encoded block: {:?}", encoded_block);
    println!(
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
    println!("Parsed loop instruction: {:?}", loop_instr);
    println!("Bytes read: {}", loop_bytes_read);

    let encoded_loop = encode_instruction(&loop_instr).unwrap();
    println!("Encoded loop: {:?}", encoded_loop);
    println!("Encoding matches original: {}", encoded_loop == loop_bytes);

    // Test if instruction
    let if_bytes = vec![
        0x04, 0x40, // if with empty block type
        0x41, 0x01, // i32.const 1
        0x05, // else
        0x41, 0x00, // i32.const 0
        0x0B, // end
    ];
    let (if_instr, if_bytes_read) = parse_instruction(&if_bytes).unwrap();
    println!("Parsed if instruction: {:?}", if_instr);
    println!("Bytes read: {}", if_bytes_read);

    let encoded_if = encode_instruction(&if_instr).unwrap();
    println!("Encoded if: {:?}", encoded_if);
    println!("Encoding matches original: {}", encoded_if == if_bytes);

    // Test br_table instruction
    let br_table_bytes = vec![
        0x0E, // br_table
        0x02, // count = 2
        0x00, // label 0
        0x01, // label 1
        0x02, // default label 2
    ];
    let (br_table_instr, br_table_bytes_read) = parse_instruction(&br_table_bytes).unwrap();
    println!("Parsed br_table instruction: {:?}", br_table_instr);
    println!("Bytes read: {}", br_table_bytes_read);

    let encoded_br_table = encode_instruction(&br_table_instr).unwrap();
    println!("Encoded br_table: {:?}", encoded_br_table);
    println!(
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
    println!("Parsed nested blocks instruction: {:?}", nested_instr);
    println!("Bytes read: {}", nested_bytes_read);

    let encoded_nested = encode_instruction(&nested_instr).unwrap();
    println!("Encoded nested blocks: {:?}", encoded_nested);
    println!(
        "Encoding matches original: {}",
        encoded_nested == nested_bytes
    );
}
