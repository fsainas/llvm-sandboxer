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
    Command::new("make")
        .args(["-C", &c_files_dir_path.to_string_lossy()])
        .output()
        .expect("Failed to compile C source file");
}

/// Test one LLVM bitcode file.
fn instrument_testcase(testcase_name: &str) {

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
    
    //function.print_to_stderr();
    module.print_to_stderr();
    //println!("{:?}", module.print_to_string());
}

// This tests are manual for now, to run one of them use the following command: 
// `cargo test <test name>`.
#[test]
fn test_instrument_bad_entry_0() {
    instrument_testcase("bad_entry_0");
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
    instrument_testcase("good_entry_0");
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
    instrument_testcase("good_entry_3");
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
