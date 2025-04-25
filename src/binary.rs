#[cfg(test)]
mod tests {
    use super::*;
    // Removing unused import: use crate::section::Section;

    #[test]
    fn test_f32_roundtrip() {
    }

    #[test]
    fn test_component_binary_roundtrip() {
        // Create a basic valid component binary
        let mut binary = Vec::new();
        
        // Magic bytes
        binary.extend_from_slice(&COMPONENT_MAGIC);
        
        // Version and layer
        binary.extend_from_slice(&COMPONENT_VERSION);
        
        // Parse component
        let component = parse_component_binary(&binary).unwrap();
        
        // Generate component binary again
        let gen_binary = generate_component_binary(&component).unwrap();
        
        // Check that the generated binary matches the original
        assert_eq!(binary, gen_binary);
        
        // Test invalid binaries
        
        // Test 1: Too short
        let invalid_binary = vec![0, 1, 2];
        let result = parse_component_binary(&invalid_binary);
        assert!(result.is_err());
        
        // Test 2: Invalid magic
        let mut invalid_binary = Vec::new();
        invalid_binary.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // Wrong magic
        invalid_binary.extend_from_slice(&COMPONENT_VERSION);
        let result = parse_component_binary(&invalid_binary);
        assert!(result.is_err());
        
        // Test 3: Invalid version
        let mut invalid_binary = Vec::new();
        invalid_binary.extend_from_slice(&COMPONENT_MAGIC);
        invalid_binary.extend_from_slice(&[0x02, 0x00, 0x01, 0x00]); // Wrong version
        let result = parse_component_binary(&invalid_binary);
        assert!(result.is_err());
    }
} 