// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Basic tests for wrt-debug

#[cfg(test)]
mod tests {
    use crate::{AbbreviationTable, DwarfDebugInfo, LineInfo};

    #[test]
    fn test_create_debug_info() {
        let module_bytes = &[0u8; 100];
        let debug_info = DwarfDebugInfo::new(module_bytes);
        assert!(!debug_info.has_debug_info());
    }

    #[test]
    fn test_add_section() {
        let module_bytes = &[0u8; 100];
        let mut debug_info = DwarfDebugInfo::new(module_bytes);

        debug_info.add_section(".debug_line", 10, 20);
        assert!(debug_info.has_debug_info());
    }

    #[test]
    fn test_add_multiple_sections() {
        let module_bytes = &[0u8; 1000];
        let mut debug_info = DwarfDebugInfo::new(module_bytes);

        debug_info.add_section(".debug_info", 100, 200);
        debug_info.add_section(".debug_abbrev", 300, 100);
        debug_info.add_section(".debug_line", 400, 150);
        debug_info.add_section(".debug_str", 550, 50);

        assert!(debug_info.has_debug_info());
    }

    #[test]
    #[cfg(feature = "abbrev")]
    fn test_abbreviation_table() {
        let mut abbrev_table = AbbreviationTable::new();
        assert!(abbrev_table.is_empty());
        assert_eq!(abbrev_table.len(), 0);

        // Test finding non-existent abbreviation
        assert!(abbrev_table.find(1).is_none());
    }

    #[test]
    fn test_cursor_basic_operations() {
        use crate::DwarfCursor;

        let data = &[0x01, 0x02, 0x03, 0x04, 0x05];
        let mut cursor = DwarfCursor::new(data);

        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.remaining(), 5);
        assert!(!cursor.is_at_end());

        // Read a byte
        let byte = cursor.read_u8().unwrap();
        assert_eq!(byte, 0x01);
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.remaining(), 4);

        // Skip some bytes
        cursor.skip(2).unwrap();
        assert_eq!(cursor.position(), 3);
        assert_eq!(cursor.remaining(), 2);

        // Read remaining bytes
        let remaining = cursor.read_bytes(2).unwrap();
        assert_eq!(remaining, &[0x04, 0x05]);
        assert!(cursor.is_at_end());
    }

    #[test]
    #[cfg(feature = "line-info")]
    fn test_line_number_state() {
        use crate::LineNumberState;

        let state = LineNumberState::new();
        // Basic construction test
        assert!(core::mem::size_of::<LineNumberState>() > 0);
    }

    #[test]
    fn test_feature_detection() {
        // Test that basic functionality works regardless of features
        let module_bytes = &[0u8; 100];
        let debug_info = DwarfDebugInfo::new(module_bytes);

        // This should always work
        assert!(!debug_info.has_debug_info());
    }

    #[test]
    #[cfg(not(feature = "line-info"))]
    fn test_without_line_info() {
        // Test that we can still create debug info without line-info feature
        let module_bytes = &[0u8; 100];
        let mut debug_info = DwarfDebugInfo::new(module_bytes);

        debug_info.add_section(".debug_info", 100, 200);
        // Without line-info, this might still be false unless debug-info is enabled
        let _has_debug = debug_info.has_debug_info();
    }

    #[test]
    #[cfg(not(feature = "debug-info"))]
    fn test_without_debug_info() {
        // Test that we can still create debug info without debug-info feature
        let module_bytes = &[0u8; 100];
        let mut debug_info = DwarfDebugInfo::new(module_bytes);

        debug_info.add_section(".debug_line", 100, 200);
        // With line-info but without debug-info, this should work if line-info is
        // enabled
        let _has_debug = debug_info.has_debug_info();
    }
}

// Runtime debug feature tests
#[cfg(test)]
#[cfg(feature = "runtime-inspection")]
mod runtime_tests {
    #[test]
    #[cfg(feature = "runtime-variables")]
    fn test_variable_formatting() {
        use crate::{parameter::BasicType, runtime_api::VariableValue, runtime_vars::ValueDisplay};

        let value = VariableValue {
            bytes: [42, 0, 0, 0, 0, 0, 0, 0],
            size: 4,
            var_type: BasicType::SignedInt(4),
            address: None,
        };

        let mut output = String::new();
        ValueDisplay { value: &value }
            .display(|s| {
                output.push_str(s);
                Ok(())
            })
            .unwrap();

        assert_eq!(output, "42");
    }

    #[test]
    #[cfg(feature = "runtime-memory")]
    fn test_memory_regions() {
        use crate::runtime_memory::{MemoryInspector, MemoryRegion, MemoryRegionType};

        let mut inspector = MemoryInspector::new();

        inspector
            .add_region(MemoryRegion {
                start: 0x1000,
                size: 0x1000,
                region_type: MemoryRegionType::LinearMemory,
                writable: true,
                name: "test",
            })
            .unwrap();

        assert!(inspector.find_region(0x1500).is_some());
        assert!(inspector.find_region(0x500).is_none());
    }

    #[test]
    #[cfg(feature = "runtime-breakpoints")]
    fn test_breakpoint_basic() {
        use crate::runtime_break::BreakpointManager;

        let mut manager = BreakpointManager::new();
        let id = manager.add_breakpoint(0x1000).unwrap();

        assert!(manager.find_by_address(0x1000).is_some());

        manager.remove_breakpoint(id).unwrap();
        assert!(manager.find_by_address(0x1000).is_none());
    }

    #[test]
    #[cfg(feature = "runtime-stepping")]
    fn test_step_modes() {
        use crate::runtime_step::{StepController, StepMode};

        let controller = StepController::new();
        assert_eq!(controller.mode(), StepMode::None);
    }

    #[test]
    fn test_debug_action() {
        use crate::runtime_api::DebugAction;

        // Just verify the enum exists and can be used
        let action = DebugAction::Continue;
        match action {
            DebugAction::Continue => {}
            _ => panic!("Wrong action"),
        }
    }
}
