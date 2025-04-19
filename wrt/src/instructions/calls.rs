/// Executes a `call_indirect` instruction.
pub fn call_indirect(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &mut StacklessEngine,
    type_idx: u32,
    table_idx: u32,
) -> Result<ControlFlow> {
    let func_table_idx = stack.pop_i32()?;
    let current_instance_idx = frame.instance_idx();

    // Find the function index from the table
    let func_idx = engine.with_instance(current_instance_idx as usize, |instance| {
        let table = instance.get_table(table_idx)?;
        let func_ref = table
            .get(func_table_idx as u32)
            .ok_or_else(|| Error::new(kinds::TableAccessOutOfBounds))? // Use specific error
            .ok_or_else(|| Error::new(kinds::InvalidFunctionReference))?; // Uninitialized element

        match func_ref {
            Value::FuncRef(Some(idx)) => Ok(idx), // Dereference Option<u32>
            Value::FuncRef(None) => Err(Error::new(kinds::InvalidFunctionReference)), // Null reference
            _ => Err(Error::new(kinds::TypeMismatch(
                "Expected FuncRef in table".to_string(),
            ))),
        }
    })?;

    // Resolve the actual function address using the engine
    let func_addr = engine.resolve_func_addr(current_instance_idx, func_idx)?;
    let expected_type = engine.get_function_type_by_index(type_idx)?;
    let actual_type = engine.resolve_func_type(current_instance_idx, func_idx)?;

    // Check signature match
    if expected_type != actual_type {
        return Err(Error::new(kinds::InvalidFunctionTypeError(
            format!("Indirect call type mismatch: expected {:?}, found {:?}", expected_type, actual_type)
        )));
    }

    // Prepare arguments
    let mut args = Vec::with_capacity(actual_type.params().len());
    for _ in 0..actual_type.params().len() {
        args.push(stack.pop()?);
    }
    args.reverse(); // Pop order is reverse of call order

    Ok(ControlFlow::Call { func_addr, args })
}




