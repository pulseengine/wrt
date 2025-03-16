use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wrt::{
    new_engine, new_stackless_engine, BlockType, FuncType, Function, Instruction, Module, Value,
    ValueType,
};

fn create_test_module() -> Module {
    let mut module = Module::new();

    // Add a simple function type (i32, i32) -> i32
    let func_type = FuncType {
        params: vec![ValueType::I32, ValueType::I32],
        results: vec![ValueType::I32],
    };
    module.types.push(func_type);

    // Add a simple add function
    let function = Function {
        type_idx: 0,
        locals: vec![],
        body: vec![
            Instruction::LocalGet(0),
            Instruction::LocalGet(1),
            Instruction::I32Add,
        ],
    };
    module.functions.push(function);

    module
}

fn create_complex_module() -> Module {
    let mut module = Module::new();

    // Function type (i32) -> i32
    let func_type = FuncType {
        params: vec![ValueType::I32],
        results: vec![ValueType::I32],
    };
    module.types.push(func_type);

    // Create a recursive countdown function that adds numbers from n down to 0
    let function = Function {
        type_idx: 0,
        locals: vec![],
        body: vec![
            // if n == 0 return 0
            Instruction::LocalGet(0),
            Instruction::I32Const(0),
            Instruction::I32Eq,
            Instruction::If(BlockType::Type(ValueType::I32)),
            Instruction::I32Const(0),
            Instruction::Else,
            // else return n + countdown(n-1)
            Instruction::LocalGet(0),
            Instruction::I32Const(1),
            Instruction::I32Sub,
            Instruction::Call(0),
            Instruction::LocalGet(0),
            Instruction::I32Add,
            Instruction::End,
        ],
    };
    module.functions.push(function);

    module
}

fn create_memory_module() -> Module {
    let mut module = Module::new();

    // Function type (i32, i32) -> i32
    let func_type = FuncType {
        params: vec![ValueType::I32, ValueType::I32],
        results: vec![ValueType::I32],
    };
    module.types.push(func_type);

    // Create a function that performs memory operations
    let function = Function {
        type_idx: 0,
        locals: vec![ValueType::I32], // Local variable for sum
        body: vec![
            // Initialize sum to 0
            Instruction::I32Const(0),
            Instruction::LocalSet(2),
            // Store first parameter at address 0
            Instruction::I32Const(0),
            Instruction::LocalGet(0),
            Instruction::I32Store(0, 0),
            // Store second parameter at address 4
            Instruction::I32Const(4),
            Instruction::LocalGet(1),
            Instruction::I32Store(0, 0),
            // Load both values and add them
            Instruction::I32Const(0),
            Instruction::I32Load(0, 0),
            Instruction::I32Const(4),
            Instruction::I32Load(0, 0),
            Instruction::I32Add,
        ],
    };
    module.functions.push(function);

    module
}

fn benchmark_engine_loading(c: &mut Criterion) {
    let module = create_test_module();

    let mut group = c.benchmark_group("wasm_component_loading");

    group.bench_function("normal_engine", |b| {
        b.iter(|| {
            let mut engine = new_engine();
            black_box(engine.instantiate(module.clone())).unwrap();
        });
    });

    group.bench_function("stackless_engine", |b| {
        b.iter(|| {
            let mut engine = new_stackless_engine();
            black_box(engine.instantiate(module.clone())).unwrap();
        });
    });

    group.finish();
}

fn benchmark_simple_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple Module Execution");

    // Create a simple module that adds two numbers
    let module = create_test_module();

    group.bench_function("normal_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_engine();
            engine.instantiate(module.clone()).unwrap();
            engine
                .execute(0, 0, vec![Value::I32(5), Value::I32(3)])
                .unwrap()
        })
    });

    group.bench_function("stackless_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_stackless_engine();
            let instance_idx = engine.instantiate(module.clone()).unwrap();
            engine
                .execute(instance_idx, 0, vec![Value::I32(5), Value::I32(3)])
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_complex_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complex Module Execution");

    // Create a complex module with multiple functions and control flow
    let module = create_complex_module();

    group.bench_function("normal_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_engine();
            engine.instantiate(module.clone()).unwrap();
            engine.execute(0, 0, vec![Value::I32(10)]).unwrap()
        })
    });

    group.bench_function("stackless_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_stackless_engine();
            let instance_idx = engine.instantiate(module.clone()).unwrap();
            engine
                .execute(instance_idx, 0, vec![Value::I32(10)])
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Operations");

    // Create a module that performs memory operations
    let module = create_memory_module();

    group.bench_function("normal_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_engine();
            engine.instantiate(module.clone()).unwrap();
            engine.execute(0, 0, vec![]).unwrap()
        })
    });

    group.bench_function("stackless_engine", |b| {
        b.iter(|| {
            let mut engine = wrt::new_stackless_engine();
            let instance_idx = engine.instantiate(module.clone()).unwrap();
            engine.execute(instance_idx, 0, vec![]).unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_engine_loading,
    benchmark_simple_execution,
    benchmark_complex_execution,
    benchmark_memory_operations
);
criterion_main!(benches);
