use llvm_sandboxer::runtime;

use std::path::Path;
use std::process::Command;
use inkwell::module::Module;
use inkwell::context::Context;
//use inkwell::values::FunctionValue;

/// Compile testcases' C sources.
fn compile_c_files() {
    let c_files_dir_path = Path::new("tests/c_files/");

    // Compile the C source file
    let _ = Command::new("make")
        .args(["-C", &c_files_dir_path.to_string_lossy()])
        .output()
        .expect("Failed to compile C source file.");
}

/// Compile instrumented LLVMs to bitcode
fn compile_ll(filepath: &str) -> String {
    let filepath = Path::new(filepath);
    let exec_filepath = filepath.with_extension("o");
    println!("{:?}", exec_filepath);

    // Compile
    let _ = Command::new("clang")
        .arg(filepath)
        .arg("-o")
        .arg(exec_filepath.clone())
        .output()
        .expect("Failed to compile LLVMs to bitcode.");

    exec_filepath.display().to_string()
}

/// Test one LLVM bitcode file.
fn instrument_testcase(testcase_name: &str) -> String {

    // Get testcase bitcode path
    let bitcode_path = format!("target/tests/{}.bc", testcase_name);
    let bitcode_path = Path::new(&bitcode_path);
    
    // Compile if it not exists
    if !bitcode_path.exists() {
        compile_c_files();
    }

    // Parse bitcode
    let context = Context::create();
    let module = Module::parse_bitcode_from_path(&bitcode_path, &context).unwrap();

    if let Err(err) = runtime::instrument(testcase_name, &context, &module) {
        println!("Error occurred: {:?}", err);
    } else {
        println!("Instrumentation completed successfully");
    }

    // Retrive function value
    let function = module.get_function(testcase_name).unwrap();
    
    // Save to file
    let filepath = format!("target/tests/instrumented/{}_instr.ll", testcase_name);
    let _ = module.print_to_file(filepath.clone());

    return filepath
}

// This tests are manual for now, to run one of them use the following command: 
// `cargo test <test name>`.
#[test]
fn test_instrument_bad_entry_0() {
    let ll_filepath = instrument_testcase("bad_entry_0");
    let filepath = compile_ll(&ll_filepath);
    assert_eq!(false, true);
}

#[test]
fn test_instrument_bad_entry_1() {
    instrument_testcase("bad_entry_1");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_bad_entry_2() {
    instrument_testcase("bad_entry_2");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_bad_entry_3() {
    instrument_testcase("bad_entry_3");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_0() {
    let ll_filepath = instrument_testcase("good_entry_0");
    let filepath = compile_ll(&ll_filepath);
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_1() {
    instrument_testcase("good_entry_1");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_2() {
    instrument_testcase("good_entry_2");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_3() {
    let ll_filepath = instrument_testcase("good_entry_3");
    let filepath = compile_ll(&ll_filepath);
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_4() {
    instrument_testcase("good_entry_4");
    assert_eq!(false, true);
}

#[test]
fn test_instrument_good_entry_5() {
    instrument_testcase("good_entry_5");
    assert_eq!(false, true);
}
