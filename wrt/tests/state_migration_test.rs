#![cfg(feature = "serialization")]

use wrt::error::Result;
use wrt::execution::Engine;
use wrt::module::Module;
use wrt::parse::parse_module;
use wrt::values::Value;

/// This test demonstrates how to migrate WebAssembly execution state between machines.
/// In a real-world scenario, the serialized state would be transferred over a network
/// or saved to a persistent storage.
#[test]
fn test_state_migration() -> Result<()> {
    // Create a WebAssembly module with a counter
    let wasm = wat::parse_str(
        r#"
        (module
            (global (export "counter") (mut i32) (i32.const 0))
            (global (export "target") (mut i32) (i32.const 10))
            
            (func (export "set_target") (param i32)
                local.get 0
                global.set 1
            )
            
            (func (export "increment")
                global.get 0      ;; get current counter
                i32.const 1
                i32.add           ;; increment
                global.set 0      ;; set counter
            )
            
            (func (export "get_counter") (result i32)
                global.get 0
            )
            
            (func (export "get_target") (result i32)
                global.get 1
            )
        )
    "#,
    )
    .unwrap();

    // Machine 1: Initialize and run the first part
    println!("Machine 1: Initializing...");
    let module = parse_module(&wasm)?;
    let mut machine1 = Engine::new(module);

    // Set target to 10
    machine1.execute_function(0, "set_target", vec![Value::I32(10)])?;
    assert_eq!(
        machine1.execute_function(0, "get_target", vec![])?,
        vec![Value::I32(10)]
    );

    // Increment 4 times
    for i in 0..4 {
        println!("Machine 1: Increment {}", i + 1);
        machine1.execute_function(0, "increment", vec![])?;
    }

    // Check counter
    let result = machine1.execute_function(0, "get_counter", vec![])?;
    assert_eq!(result, vec![Value::I32(4)]);
    println!("Machine 1: Counter = 4, serializing state...");

    // Serialize machine state
    let serialized_state = machine1.save_state_binary()?;

    // Machine 2: Load the state and continue
    println!("Machine 2: Loading state from Machine 1...");
    let mut machine2 = Engine::load_state_binary(&serialized_state)?;

    // Verify counter state
    let result = machine2.execute_function(0, "get_counter", vec![])?;
    assert_eq!(result, vec![Value::I32(4)]);

    // Continue incrementing 3 more times
    for i in 0..3 {
        println!("Machine 2: Increment {}", i + 1);
        machine2.execute_function(0, "increment", vec![])?;
    }

    // Check counter
    let result = machine2.execute_function(0, "get_counter", vec![])?;
    assert_eq!(result, vec![Value::I32(7)]);
    println!("Machine 2: Counter = 7, serializing state...");

    // Serialize machine state
    let serialized_state = machine2.save_state_binary()?;

    // Machine 3: Load the state and continue
    println!("Machine 3: Loading state from Machine 2...");
    let mut machine3 = Engine::load_state_binary(&serialized_state)?;

    // Verify counter state
    let result = machine3.execute_function(0, "get_counter", vec![])?;
    assert_eq!(result, vec![Value::I32(7)]);

    // Increment to the target value (3 more times)
    for i in 0..3 {
        println!("Machine 3: Increment {}", i + 1);
        machine3.execute_function(0, "increment", vec![])?;
    }

    // Check if we reached the target
    let counter = machine3.execute_function(0, "get_counter", vec![])?;
    let target = machine3.execute_function(0, "get_target", vec![])?;

    assert_eq!(counter, vec![Value::I32(10)]);
    assert_eq!(target, vec![Value::I32(10)]);
    println!("Machine 3: Counter = 10, reached target!");

    Ok(())
}

/// This test demonstrates checkpointing and resuming execution after a pause.
#[test]
fn test_checkpoint_resume() -> Result<()> {
    // Create a WebAssembly module with a function that can be paused
    let wasm = wat::parse_str(
        r#"
        (module
            (global (export "sum") (mut i32) (i32.const 0))
            (global (export "i") (mut i32) (i32.const 0))
            (global (export "target") (mut i32) (i32.const 1000))
            
            (func (export "sum_to_target") (result i32)
                (local $local_sum i32)
                
                ;; Initialize sum to 0
                i32.const 0
                local.set $local_sum
                
                ;; Loop from i=0 to target-1
                loop $loop
                    ;; Get current i value
                    global.get 1 ;; i
                    
                    ;; Add i to local_sum
                    local.get $local_sum
                    global.get 1 ;; i
                    i32.add
                    local.set $local_sum
                    
                    ;; Increment i
                    global.get 1 ;; i
                    i32.const 1
                    i32.add
                    global.set 1 ;; i = i + 1
                    
                    ;; Save current sum to global
                    local.get $local_sum
                    global.set 0 ;; sum = local_sum
                    
                    ;; Check if i < target
                    global.get 1 ;; i
                    global.get 2 ;; target
                    i32.lt_s
                    br_if $loop
                end
                
                ;; Return final sum
                local.get $local_sum
            )
            
            (func (export "get_sum") (result i32)
                global.get 0
            )
            
            (func (export "get_i") (result i32)
                global.get 1
            )
        )
    "#,
    )
    .unwrap();

    // Initialize the engine with limited fuel
    println!("Initializing engine with limited fuel...");
    let module = parse_module(&wasm)?;
    let mut engine = Engine::new(module);

    // Set a fuel limit to force pausing
    engine.set_fuel(Some(1000));

    // Begin execution - this should pause due to fuel exhaustion
    println!("Starting execution (expecting to pause)...");
    match engine.execute_function(0, "sum_to_target", vec![]) {
        Ok(_) => panic!("Function completed unexpectedly"),
        Err(wrt::error::Error::FuelExhausted) => {
            println!("Engine paused as expected due to fuel exhaustion");
        }
        Err(e) => return Err(e),
    }

    // Create a checkpoint at this paused state
    println!("Creating checkpoint at paused state...");
    let checkpoint = engine.create_checkpoint()?;

    // Check current values
    let current_i = engine.execute_function(0, "get_i", vec![])?;
    let current_sum = engine.execute_function(0, "get_sum", vec![])?;
    println!(
        "Current i: {}, sum: {}",
        current_i[0].as_i32().unwrap(),
        current_sum[0].as_i32().unwrap()
    );

    assert!(current_i[0].as_i32().unwrap() > 0, "i should have advanced");
    assert!(
        current_sum[0].as_i32().unwrap() > 0,
        "sum should have accumulated"
    );

    // Try to continue execution with limited fuel - will pause again
    println!("Adding limited fuel and trying to continue...");
    engine.set_fuel(Some(1000));
    match engine.resume() {
        Ok(_) => panic!("Function completed unexpectedly"),
        Err(wrt::error::Error::FuelExhausted) => {
            println!("Engine paused again due to fuel exhaustion");
        }
        Err(e) => return Err(e),
    }

    // Restore from the first checkpoint
    println!("Restoring from checkpoint...");
    let mut restored_engine = Engine::restore_from_checkpoint(&checkpoint)?;

    // Now add unlimited fuel and resume
    println!("Adding unlimited fuel and resuming execution...");
    restored_engine.set_fuel(None);
    let result = restored_engine.resume()?;

    // We should have computed the sum of integers from 0 to 999
    let expected = (0..1000).sum::<i32>();
    assert_eq!(result, vec![Value::I32(expected)]);
    println!("Successfully completed execution, result: {}", expected);

    Ok(())
}
