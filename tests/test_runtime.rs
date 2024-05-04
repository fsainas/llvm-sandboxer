use llvm_sandboxer::runtime;

use std::path::Path;
use std::process::Command;
use inkwell::module::Module;
use inkwell::context::Context;

/// Compile testcases' C sources.
fn compile_c_files() {
    let c_files_dir_path = Path::new("tests/c_files/");

    // Compile the C source file
    Command::new("make")
        .args(["-C", &c_files_dir_path.to_string_lossy()])
        .output()
        .expect("Failed to compile C source file.");
}

/// Compile instrumented LLVMs to executable
fn compile_ll_to_exec(filepath: &str) -> String {
    let filepath = Path::new(filepath);
    let exec_filepath = filepath.with_extension("o");

    // Compile
    let output = Command::new("clang")
        .arg(filepath)
        .arg("-o")
        .arg(exec_filepath.clone())
        .output()
        .expect("Failed to compile LLVMs to executable.");

    println!("{:?}", output);

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

    match runtime::instrument(testcase_name, &context, &module, true) {
        Ok(()) => println!("Instrumentation completed successfully"),
        Err(err) => println!("Error occurred: {:?}", err)
    }

    // Save to file
    let filepath = format!("target/tests/instrumented/{}_instrumented.ll", testcase_name);
    let _ = module.print_to_file(filepath.clone());

    return filepath
}

// This tests are manual for now, to run one of them use the following command: 
// `cargo test <test name>`.
#[test]
fn test_instrument_bad_entry_0() {
    let test_case_name = "bad_entry_0";
    let ll_filepath = instrument_testcase(test_case_name);
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it crashes
    assert_eq!(output.status.code(), None);
}

#[test]
fn test_instrument_bad_entry_1() {
    let ll_filepath = instrument_testcase("bad_entry_1");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it crashes
    assert_eq!(output.status.code(), None);
}

#[test]
fn test_instrument_bad_entry_2() {
    let ll_filepath = instrument_testcase("bad_entry_2");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it crashes
    assert_eq!(output.status.code(), None);
}

#[test]
fn test_instrument_bad_entry_3() {
    let ll_filepath = instrument_testcase("bad_entry_3");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it crashes
    assert_eq!(output.status.code(), None);
}

#[test]
fn test_instrument_good_entry_0() {
    let ll_filepath = instrument_testcase("good_entry_0");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_good_entry_1() {
    let ll_filepath = instrument_testcase("good_entry_1");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_good_entry_2() {
    let ll_filepath = instrument_testcase("good_entry_2");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}.", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_good_entry_3() {
    let ll_filepath = instrument_testcase("good_entry_3");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_good_entry_4() {
    let ll_filepath = instrument_testcase("good_entry_4");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_good_entry_5() {
    let ll_filepath = instrument_testcase("good_entry_5");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}", filepath));

    // Check that it doesn't crash
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_instrument_phi_0() {
    let ll_filepath = instrument_testcase("phi_0");
    let filepath = compile_ll_to_exec(&ll_filepath);

    // Execute the instrumented testcase
    let output = Command::new(filepath.clone())
        .output()
        .expect(&format!("Cannot execute {}", filepath));

    assert_eq!(output.status.code(), Some(0));
}