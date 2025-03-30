use std::fs;
use std::path::Path;
use wast::core::{NanPattern, WastArgCore, WastRetCore};
use wast::{
    parser::{self, ParseBuffer},
    Wast, WastArg, WastDirective, WastExecute, WastRet,
};
use wrt::{Error, ExecutionEngine as Engine, Module, Value};

fn convert_wast_arg_core(arg: &WastArg) -> Result<Value, Error> {
    match arg {
        WastArg::Core(core_arg) => match core_arg {
            WastArgCore::I32(x) => Ok(Value::I32(*x)),
            WastArgCore::I64(x) => Ok(Value::I64(*x)),
            WastArgCore::F32(x) => Ok(Value::F32(f32::from_bits(x.bits))),
            WastArgCore::F64(x) => Ok(Value::F64(f64::from_bits(x.bits))),
            _ => Err(Error::Validation("Unsupported argument type".into())),
        },
        _ => Err(Error::Validation("Unsupported argument type".into())),
    }
}

fn convert_wast_ret_core(ret: &WastRet) -> Result<Value, Error> {
    match ret {
        WastRet::Core(core_ret) => match core_ret {
            WastRetCore::I32(x) => Ok(Value::I32(*x)),
            WastRetCore::I64(x) => Ok(Value::I64(*x)),
            WastRetCore::F32(x) => match x {
                NanPattern::Value(x) => Ok(Value::F32(f32::from_bits(x.bits))),
                NanPattern::CanonicalNan => Ok(Value::F32(f32::NAN)),
                NanPattern::ArithmeticNan => Ok(Value::F32(f32::NAN)),
            },
            WastRetCore::F64(x) => match x {
                NanPattern::Value(x) => Ok(Value::F64(f64::from_bits(x.bits))),
                NanPattern::CanonicalNan => Ok(Value::F64(f64::NAN)),
                NanPattern::ArithmeticNan => Ok(Value::F64(f64::NAN)),
            },
            _ => Err(Error::Validation("Unsupported return type".into())),
        },
        _ => Err(Error::Validation("Unsupported return type".into())),
    }
}

fn test_wast_directive(engine: &mut Engine, directive: &mut WastDirective) -> Result<(), Error> {
    match directive {
        WastDirective::Module(ref mut wast_module) => {
            // Get the binary from the WAST module
            let binary = wast_module
                .encode()
                .map_err(|e| Error::Parse(e.to_string()))?;

            // Debug output
            println!("Binary: {:02x?}", binary);

            // Create and load the WRT module
            let mut wrt_module = Module::new()?;
            let loaded_module = wrt_module.load_from_binary(&binary)?;

            // Debug output
            println!("Module exports: {:?}", loaded_module.exports);

            // Instantiate the module
            let instance_idx = engine.instantiate(loaded_module)?;
            println!(
                "DEBUG: instantiate called for module with instance index {}",
                instance_idx
            );

            Ok(())
        }
        WastDirective::AssertReturn {
            span: _,
            exec,
            results,
        } => {
            match exec {
                WastExecute::Invoke(invoke) => {
                    let args: Result<Vec<Value>, _> =
                        invoke.args.iter().map(convert_wast_arg_core).collect();
                    let args = args?;
                    println!("DEBUG: Invoking {} with args: {:?}", invoke.name, args);

                    let expected: Result<Vec<Value>, _> =
                        results.iter().map(convert_wast_ret_core).collect();
                    let expected = expected?;
                    println!("DEBUG: Expected result: {:?}", expected);

                    // Execute the function and compare results
                    let actual = engine.invoke_export(invoke.name, &args)?;
                    println!("DEBUG: Actual result: {:?}", actual);
                    println!(
                        "DEBUG: Comparison: actual == expected is {}",
                        actual == expected
                    );
                    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
                        println!(
                            "DEBUG: Result[{}]: actual={:?}, expected={:?}, match={}",
                            i,
                            a,
                            e,
                            a == e
                        );
                    }

                    assert_eq!(
                        actual, expected,
                        "Function {} returned unexpected results",
                        invoke.name
                    );
                    Ok(())
                }
                _ => Ok(()), // Skip other types of executions for now
            }
        }
        _ => Ok(()), // Skip other directives for now
    }
}

fn test_wast_file(path: &Path) -> Result<(), Error> {
    let contents = fs::read_to_string(path)
        .map_err(|e| Error::Parse(format!("Failed to read file: {}", e)))?;

    let buf = ParseBuffer::new(&contents)
        .map_err(|e| Error::Parse(format!("Failed to create parse buffer: {}", e)))?;

    let wast: Wast =
        parser::parse(&buf).map_err(|e| Error::Parse(format!("Failed to parse WAST: {}", e)))?;

    let module = Module::new()?;
    let mut engine = Engine::new(module);
    for mut directive in wast.directives {
        test_wast_directive(&mut engine, &mut directive)?;
    }

    Ok(())
}

#[test]
fn test_simple_add() -> Result<(), Error> {
    // Create a simple WAST test case inline
    let wast_content = r#"
;; i32 operations

(module
  (func (export "add") (param $x i32) (param $y i32) (result i32) (i32.add (local.get $x) (local.get $y)))
  (func (export "sub") (param $x i32) (param $y i32) (result i32) (i32.sub (local.get $x) (local.get $y)))
  (func (export "mul") (param $x i32) (param $y i32) (result i32) (i32.mul (local.get $x) (local.get $y)))
  (func (export "div_s") (param $x i32) (param $y i32) (result i32) (i32.div_s (local.get $x) (local.get $y)))
  (func (export "div_u") (param $x i32) (param $y i32) (result i32) (i32.div_u (local.get $x) (local.get $y)))
  (func (export "rem_s") (param $x i32) (param $y i32) (result i32) (i32.rem_s (local.get $x) (local.get $y)))
  (func (export "rem_u") (param $x i32) (param $y i32) (result i32) (i32.rem_u (local.get $x) (local.get $y)))
  (func (export "and") (param $x i32) (param $y i32) (result i32) (i32.and (local.get $x) (local.get $y)))
  (func (export "or") (param $x i32) (param $y i32) (result i32) (i32.or (local.get $x) (local.get $y)))
  (func (export "xor") (param $x i32) (param $y i32) (result i32) (i32.xor (local.get $x) (local.get $y)))
  (func (export "shl") (param $x i32) (param $y i32) (result i32) (i32.shl (local.get $x) (local.get $y)))
  (func (export "shr_s") (param $x i32) (param $y i32) (result i32) (i32.shr_s (local.get $x) (local.get $y)))
  (func (export "shr_u") (param $x i32) (param $y i32) (result i32) (i32.shr_u (local.get $x) (local.get $y)))
  (func (export "rotl") (param $x i32) (param $y i32) (result i32) (i32.rotl (local.get $x) (local.get $y)))
  (func (export "rotr") (param $x i32) (param $y i32) (result i32) (i32.rotr (local.get $x) (local.get $y)))
  (func (export "clz") (param $x i32) (result i32) (i32.clz (local.get $x)))
  (func (export "ctz") (param $x i32) (result i32) (i32.ctz (local.get $x)))
  (func (export "popcnt") (param $x i32) (result i32) (i32.popcnt (local.get $x)))
  (func (export "extend8_s") (param $x i32) (result i32) (i32.extend8_s (local.get $x)))
  (func (export "extend16_s") (param $x i32) (result i32) (i32.extend16_s (local.get $x)))
  (func (export "eqz") (param $x i32) (result i32) (i32.eqz (local.get $x)))
  (func (export "eq") (param $x i32) (param $y i32) (result i32) (i32.eq (local.get $x) (local.get $y)))
  (func (export "ne") (param $x i32) (param $y i32) (result i32) (i32.ne (local.get $x) (local.get $y)))
  (func (export "lt_s") (param $x i32) (param $y i32) (result i32) (i32.lt_s (local.get $x) (local.get $y)))
  (func (export "lt_u") (param $x i32) (param $y i32) (result i32) (i32.lt_u (local.get $x) (local.get $y)))
  (func (export "le_s") (param $x i32) (param $y i32) (result i32) (i32.le_s (local.get $x) (local.get $y)))
  (func (export "le_u") (param $x i32) (param $y i32) (result i32) (i32.le_u (local.get $x) (local.get $y)))
  (func (export "gt_s") (param $x i32) (param $y i32) (result i32) (i32.gt_s (local.get $x) (local.get $y)))
  (func (export "gt_u") (param $x i32) (param $y i32) (result i32) (i32.gt_u (local.get $x) (local.get $y)))
  (func (export "ge_s") (param $x i32) (param $y i32) (result i32) (i32.ge_s (local.get $x) (local.get $y)))
  (func (export "ge_u") (param $x i32) (param $y i32) (result i32) (i32.ge_u (local.get $x) (local.get $y)))
)

(assert_return (invoke "add" (i32.const 1) (i32.const 1)) (i32.const 2))
(assert_return (invoke "add" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "add" (i32.const -1) (i32.const -1)) (i32.const -2))
(assert_return (invoke "add" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "add" (i32.const 0x7fffffff) (i32.const 1)) (i32.const 0x80000000))
(assert_return (invoke "add" (i32.const 0x80000000) (i32.const -1)) (i32.const 0x7fffffff))
(assert_return (invoke "add" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "add" (i32.const 0x3fffffff) (i32.const 1)) (i32.const 0x40000000))

(assert_return (invoke "sub" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "sub" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "sub" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "sub" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "sub" (i32.const 0x80000000) (i32.const 1)) (i32.const 0x7fffffff))
(assert_return (invoke "sub" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "sub" (i32.const 0x3fffffff) (i32.const -1)) (i32.const 0x40000000))

(assert_return (invoke "mul" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "mul" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "mul" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "mul" (i32.const 0x10000000) (i32.const 4096)) (i32.const 0))
(assert_return (invoke "mul" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "mul" (i32.const 0x80000000) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "mul" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0x80000001))
(assert_return (invoke "mul" (i32.const 0x01234567) (i32.const 0x76543210)) (i32.const 0x358e7470))
(assert_return (invoke "mul" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))

(assert_trap (invoke "div_s" (i32.const 1) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "div_s" (i32.const 0) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "div_s" (i32.const 0x80000000) (i32.const -1)) "integer overflow")
(assert_trap (invoke "div_s" (i32.const 0x80000000) (i32.const 0)) "integer divide by zero")
(assert_return (invoke "div_s" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "div_s" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "div_s" (i32.const 0) (i32.const -1)) (i32.const 0))
(assert_return (invoke "div_s" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "div_s" (i32.const 0x80000000) (i32.const 2)) (i32.const 0xc0000000))
(assert_return (invoke "div_s" (i32.const 0x80000001) (i32.const 1000)) (i32.const 0xffdf3b65))
(assert_return (invoke "div_s" (i32.const 5) (i32.const 2)) (i32.const 2))
(assert_return (invoke "div_s" (i32.const -5) (i32.const 2)) (i32.const -2))
(assert_return (invoke "div_s" (i32.const 5) (i32.const -2)) (i32.const -2))
(assert_return (invoke "div_s" (i32.const -5) (i32.const -2)) (i32.const 2))
(assert_return (invoke "div_s" (i32.const 7) (i32.const 3)) (i32.const 2))
(assert_return (invoke "div_s" (i32.const -7) (i32.const 3)) (i32.const -2))
(assert_return (invoke "div_s" (i32.const 7) (i32.const -3)) (i32.const -2))
(assert_return (invoke "div_s" (i32.const -7) (i32.const -3)) (i32.const 2))
(assert_return (invoke "div_s" (i32.const 11) (i32.const 5)) (i32.const 2))
(assert_return (invoke "div_s" (i32.const 17) (i32.const 7)) (i32.const 2))

(assert_trap (invoke "div_u" (i32.const 1) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "div_u" (i32.const 0) (i32.const 0)) "integer divide by zero")
(assert_return (invoke "div_u" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "div_u" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "div_u" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "div_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "div_u" (i32.const 0x80000000) (i32.const 2)) (i32.const 0x40000000))
(assert_return (invoke "div_u" (i32.const 0x8ff00ff0) (i32.const 0x10001)) (i32.const 0x8fef))
(assert_return (invoke "div_u" (i32.const 0x80000001) (i32.const 1000)) (i32.const 0x20c49b))
(assert_return (invoke "div_u" (i32.const 5) (i32.const 2)) (i32.const 2))
(assert_return (invoke "div_u" (i32.const -5) (i32.const 2)) (i32.const 0x7ffffffd))
(assert_return (invoke "div_u" (i32.const 5) (i32.const -2)) (i32.const 0))
(assert_return (invoke "div_u" (i32.const -5) (i32.const -2)) (i32.const 0))
(assert_return (invoke "div_u" (i32.const 7) (i32.const 3)) (i32.const 2))
(assert_return (invoke "div_u" (i32.const 11) (i32.const 5)) (i32.const 2))
(assert_return (invoke "div_u" (i32.const 17) (i32.const 7)) (i32.const 2))

(assert_trap (invoke "rem_s" (i32.const 1) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "rem_s" (i32.const 0) (i32.const 0)) "integer divide by zero")
(assert_return (invoke "rem_s" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 0) (i32.const -1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 0x80000000) (i32.const 2)) (i32.const 0))
(assert_return (invoke "rem_s" (i32.const 0x80000001) (i32.const 1000)) (i32.const -647))
(assert_return (invoke "rem_s" (i32.const 5) (i32.const 2)) (i32.const 1))
(assert_return (invoke "rem_s" (i32.const -5) (i32.const 2)) (i32.const -1))
(assert_return (invoke "rem_s" (i32.const 5) (i32.const -2)) (i32.const 1))
(assert_return (invoke "rem_s" (i32.const -5) (i32.const -2)) (i32.const -1))
(assert_return (invoke "rem_s" (i32.const 7) (i32.const 3)) (i32.const 1))
(assert_return (invoke "rem_s" (i32.const -7) (i32.const 3)) (i32.const -1))
(assert_return (invoke "rem_s" (i32.const 7) (i32.const -3)) (i32.const 1))
(assert_return (invoke "rem_s" (i32.const -7) (i32.const -3)) (i32.const -1))
(assert_return (invoke "rem_s" (i32.const 11) (i32.const 5)) (i32.const 1))
(assert_return (invoke "rem_s" (i32.const 17) (i32.const 7)) (i32.const 3))

(assert_trap (invoke "rem_u" (i32.const 1) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "rem_u" (i32.const 0) (i32.const 0)) "integer divide by zero")
(assert_return (invoke "rem_u" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "rem_u" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "rem_u" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "rem_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "rem_u" (i32.const 0x80000000) (i32.const 2)) (i32.const 0))
(assert_return (invoke "rem_u" (i32.const 0x8ff00ff0) (i32.const 0x10001)) (i32.const 0x8001))
(assert_return (invoke "rem_u" (i32.const 0x80000001) (i32.const 1000)) (i32.const 649))
(assert_return (invoke "rem_u" (i32.const 5) (i32.const 2)) (i32.const 1))
(assert_return (invoke "rem_u" (i32.const -5) (i32.const 2)) (i32.const 1))
(assert_return (invoke "rem_u" (i32.const 5) (i32.const -2)) (i32.const 5))
(assert_return (invoke "rem_u" (i32.const -5) (i32.const -2)) (i32.const -5))
(assert_return (invoke "rem_u" (i32.const 7) (i32.const 3)) (i32.const 1))
(assert_return (invoke "rem_u" (i32.const 11) (i32.const 5)) (i32.const 1))
(assert_return (invoke "rem_u" (i32.const 17) (i32.const 7)) (i32.const 3))

(assert_return (invoke "and" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "and" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "and" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "and" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "and" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "and" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0x7fffffff))
(assert_return (invoke "and" (i32.const 0xf0f0ffff) (i32.const 0xfffff0f0)) (i32.const 0xf0f0f0f0))
(assert_return (invoke "and" (i32.const 0xffffffff) (i32.const 0xffffffff)) (i32.const 0xffffffff))

(assert_return (invoke "or" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "or" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "or" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "or" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "or" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const -1))
(assert_return (invoke "or" (i32.const 0x80000000) (i32.const 0)) (i32.const 0x80000000))
(assert_return (invoke "or" (i32.const 0xf0f0ffff) (i32.const 0xfffff0f0)) (i32.const 0xffffffff))
(assert_return (invoke "or" (i32.const 0xffffffff) (i32.const 0xffffffff)) (i32.const 0xffffffff))

(assert_return (invoke "xor" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "xor" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "xor" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "xor" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "xor" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const -1))
(assert_return (invoke "xor" (i32.const 0x80000000) (i32.const 0)) (i32.const 0x80000000))
(assert_return (invoke "xor" (i32.const -1) (i32.const 0x80000000)) (i32.const 0x7fffffff))
(assert_return (invoke "xor" (i32.const -1) (i32.const 0x7fffffff)) (i32.const 0x80000000))
(assert_return (invoke "xor" (i32.const 0xf0f0ffff) (i32.const 0xfffff0f0)) (i32.const 0x0f0f0f0f))
(assert_return (invoke "xor" (i32.const 0xffffffff) (i32.const 0xffffffff)) (i32.const 0))

(assert_return (invoke "shl" (i32.const 1) (i32.const 1)) (i32.const 2))
(assert_return (invoke "shl" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "shl" (i32.const 0x7fffffff) (i32.const 1)) (i32.const 0xfffffffe))
(assert_return (invoke "shl" (i32.const 0xffffffff) (i32.const 1)) (i32.const 0xfffffffe))
(assert_return (invoke "shl" (i32.const 0x80000000) (i32.const 1)) (i32.const 0))
(assert_return (invoke "shl" (i32.const 0x40000000) (i32.const 1)) (i32.const 0x80000000))
(assert_return (invoke "shl" (i32.const 1) (i32.const 31)) (i32.const 0x80000000))
(assert_return (invoke "shl" (i32.const 1) (i32.const 32)) (i32.const 1))
(assert_return (invoke "shl" (i32.const 1) (i32.const 33)) (i32.const 2))
(assert_return (invoke "shl" (i32.const 1) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "shl" (i32.const 1) (i32.const 0x7fffffff)) (i32.const 0x80000000))

(assert_return (invoke "shr_s" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const 1)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const 0x7fffffff) (i32.const 1)) (i32.const 0x3fffffff))
(assert_return (invoke "shr_s" (i32.const 0x80000000) (i32.const 1)) (i32.const 0xc0000000))
(assert_return (invoke "shr_s" (i32.const 0x40000000) (i32.const 1)) (i32.const 0x20000000))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const 32)) (i32.const 1))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const 33)) (i32.const 0))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "shr_s" (i32.const 1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "shr_s" (i32.const 0x80000000) (i32.const 31)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const 32)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const 33)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const -1)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const 0x7fffffff)) (i32.const -1))
(assert_return (invoke "shr_s" (i32.const -1) (i32.const 0x80000000)) (i32.const -1))

(assert_return (invoke "shr_u" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const 1)) (i32.const 0x7fffffff))
(assert_return (invoke "shr_u" (i32.const 0x7fffffff) (i32.const 1)) (i32.const 0x3fffffff))
(assert_return (invoke "shr_u" (i32.const 0x80000000) (i32.const 1)) (i32.const 0x40000000))
(assert_return (invoke "shr_u" (i32.const 0x40000000) (i32.const 1)) (i32.const 0x20000000))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const 32)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const 33)) (i32.const 0))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "shr_u" (i32.const 1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const 0x80000000) (i32.const 31)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const 32)) (i32.const -1))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const 33)) (i32.const 0x7fffffff))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "shr_u" (i32.const -1) (i32.const 0x80000000)) (i32.const -1))

(assert_return (invoke "rotl" (i32.const 1) (i32.const 1)) (i32.const 2))
(assert_return (invoke "rotl" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "rotl" (i32.const -1) (i32.const 1)) (i32.const -1))
(assert_return (invoke "rotl" (i32.const 1) (i32.const 32)) (i32.const 1))
(assert_return (invoke "rotl" (i32.const 0xabcd9876) (i32.const 1)) (i32.const 0x579b30ed))
(assert_return (invoke "rotl" (i32.const 0xfe00dc00) (i32.const 4)) (i32.const 0xe00dc00f))
(assert_return (invoke "rotl" (i32.const 0xb0c1d2e3) (i32.const 5)) (i32.const 0x183a5c76))
(assert_return (invoke "rotl" (i32.const 0x00008000) (i32.const 37)) (i32.const 0x00100000))
(assert_return (invoke "rotl" (i32.const 0xb0c1d2e3) (i32.const 0xff05)) (i32.const 0x183a5c76))
(assert_return (invoke "rotl" (i32.const 0x769abcdf) (i32.const 0xffffffed)) (i32.const 0x579beed3))
(assert_return (invoke "rotl" (i32.const 0x769abcdf) (i32.const 0x8000000d)) (i32.const 0x579beed3))
(assert_return (invoke "rotl" (i32.const 1) (i32.const 31)) (i32.const 0x80000000))
(assert_return (invoke "rotl" (i32.const 0x80000000) (i32.const 1)) (i32.const 1))

(assert_return (invoke "rotr" (i32.const 1) (i32.const 1)) (i32.const 0x80000000))
(assert_return (invoke "rotr" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "rotr" (i32.const -1) (i32.const 1)) (i32.const -1))
(assert_return (invoke "rotr" (i32.const 1) (i32.const 32)) (i32.const 1))
(assert_return (invoke "rotr" (i32.const 0xff00cc00) (i32.const 1)) (i32.const 0x7f806600))
(assert_return (invoke "rotr" (i32.const 0x00080000) (i32.const 4)) (i32.const 0x00008000))
(assert_return (invoke "rotr" (i32.const 0xb0c1d2e3) (i32.const 5)) (i32.const 0x1d860e97))
(assert_return (invoke "rotr" (i32.const 0x00008000) (i32.const 37)) (i32.const 0x00000400))
(assert_return (invoke "rotr" (i32.const 0xb0c1d2e3) (i32.const 0xff05)) (i32.const 0x1d860e97))
(assert_return (invoke "rotr" (i32.const 0x769abcdf) (i32.const 0xffffffed)) (i32.const 0xe6fbb4d5))
(assert_return (invoke "rotr" (i32.const 0x769abcdf) (i32.const 0x8000000d)) (i32.const 0xe6fbb4d5))
(assert_return (invoke "rotr" (i32.const 1) (i32.const 31)) (i32.const 2))
(assert_return (invoke "rotr" (i32.const 0x80000000) (i32.const 31)) (i32.const 1))

(assert_return (invoke "clz" (i32.const 0xffffffff)) (i32.const 0))
(assert_return (invoke "clz" (i32.const 0)) (i32.const 32))
(assert_return (invoke "clz" (i32.const 0x00008000)) (i32.const 16))
(assert_return (invoke "clz" (i32.const 0xff)) (i32.const 24))
(assert_return (invoke "clz" (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "clz" (i32.const 1)) (i32.const 31))
(assert_return (invoke "clz" (i32.const 2)) (i32.const 30))
(assert_return (invoke "clz" (i32.const 0x7fffffff)) (i32.const 1))

(assert_return (invoke "ctz" (i32.const -1)) (i32.const 0))
(assert_return (invoke "ctz" (i32.const 0)) (i32.const 32))
(assert_return (invoke "ctz" (i32.const 0x00008000)) (i32.const 15))
(assert_return (invoke "ctz" (i32.const 0x00010000)) (i32.const 16))
(assert_return (invoke "ctz" (i32.const 0x80000000)) (i32.const 31))
(assert_return (invoke "ctz" (i32.const 0x7fffffff)) (i32.const 0))

(assert_return (invoke "popcnt" (i32.const -1)) (i32.const 32))
(assert_return (invoke "popcnt" (i32.const 0)) (i32.const 0))
(assert_return (invoke "popcnt" (i32.const 0x00008000)) (i32.const 1))
(assert_return (invoke "popcnt" (i32.const 0x80008000)) (i32.const 2))
(assert_return (invoke "popcnt" (i32.const 0x7fffffff)) (i32.const 31))
(assert_return (invoke "popcnt" (i32.const 0xAAAAAAAA)) (i32.const 16))
(assert_return (invoke "popcnt" (i32.const 0x55555555)) (i32.const 16))
(assert_return (invoke "popcnt" (i32.const 0xDEADBEEF)) (i32.const 24))

(assert_return (invoke "extend8_s" (i32.const 0)) (i32.const 0))
(assert_return (invoke "extend8_s" (i32.const 0x7f)) (i32.const 127))
(assert_return (invoke "extend8_s" (i32.const 0x80)) (i32.const -128))
(assert_return (invoke "extend8_s" (i32.const 0xff)) (i32.const -1))
(assert_return (invoke "extend8_s" (i32.const 0x012345_00)) (i32.const 0))
(assert_return (invoke "extend8_s" (i32.const 0xfedcba_80)) (i32.const -0x80))
(assert_return (invoke "extend8_s" (i32.const -1)) (i32.const -1))

(assert_return (invoke "extend16_s" (i32.const 0)) (i32.const 0))
(assert_return (invoke "extend16_s" (i32.const 0x7fff)) (i32.const 32767))
(assert_return (invoke "extend16_s" (i32.const 0x8000)) (i32.const -32768))
(assert_return (invoke "extend16_s" (i32.const 0xffff)) (i32.const -1))
(assert_return (invoke "extend16_s" (i32.const 0x0123_0000)) (i32.const 0))
(assert_return (invoke "extend16_s" (i32.const 0xfedc_8000)) (i32.const -0x8000))
(assert_return (invoke "extend16_s" (i32.const -1)) (i32.const -1))

(assert_return (invoke "eqz" (i32.const 0)) (i32.const 1))
(assert_return (invoke "eqz" (i32.const 1)) (i32.const 0))
(assert_return (invoke "eqz" (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "eqz" (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "eqz" (i32.const 0xffffffff)) (i32.const 0))

(assert_return (invoke "eq" (i32.const 0) (i32.const 0)) (i32.const 1))
(assert_return (invoke "eq" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "eq" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "eq" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "eq" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "eq" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "eq" (i32.const -1) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "eq" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))

(assert_return (invoke "ne" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "ne" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "ne" (i32.const -1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "ne" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "ne" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "ne" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0x80000000) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0x80000000) (i32.const -1)) (i32.const 1))
(assert_return (invoke "ne" (i32.const -1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "ne" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 1))

(assert_return (invoke "lt_s" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const -1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "lt_s" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "lt_s" (i32.const 0x80000000) (i32.const 0)) (i32.const 1))
(assert_return (invoke "lt_s" (i32.const 0) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 0x80000000) (i32.const -1)) (i32.const 1))
(assert_return (invoke "lt_s" (i32.const -1) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "lt_s" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "lt_s" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))

(assert_return (invoke "lt_u" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "lt_u" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "lt_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 1))
(assert_return (invoke "lt_u" (i32.const -1) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "lt_u" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 1))

(assert_return (invoke "le_s" (i32.const 0) (i32.const 0)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const -1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "le_s" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 0x80000000) (i32.const 0)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 0) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "le_s" (i32.const 0x80000000) (i32.const -1)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const -1) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "le_s" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "le_s" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))

(assert_return (invoke "le_u" (i32.const 0) (i32.const 0)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "le_u" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "le_u" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "le_u" (i32.const 0) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 1))
(assert_return (invoke "le_u" (i32.const -1) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "le_u" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "le_u" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 1))

(assert_return (invoke "gt_s" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "gt_s" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 0) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "gt_s" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const -1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "gt_s" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "gt_s" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 1))

(assert_return (invoke "gt_u" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const -1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "gt_u" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const -1) (i32.const -1)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "gt_u" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const 0x80000000) (i32.const 0)) (i32.const 1))
(assert_return (invoke "gt_u" (i32.const 0) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "gt_u" (i32.const -1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "gt_u" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "gt_u" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))

(assert_return (invoke "ge_s" (i32.const 0) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "ge_s" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "ge_s" (i32.const 0x80000000) (i32.const 0)) (i32.const 0))
(assert_return (invoke "ge_s" (i32.const 0) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "ge_s" (i32.const -1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ge_s" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 0))
(assert_return (invoke "ge_s" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 1))

(assert_return (invoke "ge_u" (i32.const 0) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const -1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0x7fffffff) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0) (i32.const 1)) (i32.const 0))
(assert_return (invoke "ge_u" (i32.const 0x80000000) (i32.const 0)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0) (i32.const 0x80000000)) (i32.const 0))
(assert_return (invoke "ge_u" (i32.const 0x80000000) (i32.const -1)) (i32.const 0))
(assert_return (invoke "ge_u" (i32.const -1) (i32.const 0x80000000)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0x80000000) (i32.const 0x7fffffff)) (i32.const 1))
(assert_return (invoke "ge_u" (i32.const 0x7fffffff) (i32.const 0x80000000)) (i32.const 0))
    "#;

    // Write the test case to a temporary file
    let dir = tempfile::tempdir().unwrap();
    let test_file = dir.path().join("simple_add.wast");
    fs::write(&test_file, wast_content).unwrap();

    // Run the test
    test_wast_file(&test_file)
}

#[test]
fn test_wast_files() -> Result<(), Error> {
    let test_dir = Path::new("wrt/testsuite");
    if !test_dir.exists() {
        println!("No testsuite directory found, skipping directory tests");
        return Ok(());
    }

    for entry in fs::read_dir(test_dir)
        .map_err(|e| Error::Parse(format!("Failed to read directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| Error::Parse(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "wast") {
            println!("Testing file: {}", path.display());
            test_wast_file(&path)?;
        }
    }

    Ok(())
}
