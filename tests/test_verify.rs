use llvm_sandboxer::static_checks;

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
        .expect("Failed to compile C source file");
}

/// Test one LLVM bitcode file.
fn verify_testcase(testcase_name: &str) -> bool {
    let bitcode_path = format!("target/tests/{}.bc", testcase_name);
    let bitcode_path = Path::new(&bitcode_path);
    if !bitcode_path.exists() {
        compile_c_files();
    }
    let context = Context::create();
    let module = Module::parse_bitcode_from_path(&bitcode_path, &context).unwrap();
    let function = module.get_function(testcase_name).unwrap();
    return static_checks::verify(module, function);
}

/// Test rejection of unprotected memory. 
/// bad_entry_0 has no call to utx0() or utx1(), therefore no memory protection.
#[test]
fn test_bad_entry_0() {
    assert_eq!(verify_testcase("bad_entry_0"), false);
}

#[test]
fn test_bad_entry_1() {
    assert_eq!(verify_testcase("bad_entry_1"), false);
}

#[test]
fn test_bad_entry_2() {
    assert_eq!(verify_testcase("bad_entry_2"), false);
}

#[test]
fn test_bad_entry_3() {
    assert_eq!(verify_testcase("bad_entry_3"), false);
}

#[test]
fn test_good_entry_0() {
    assert_eq!(verify_testcase("good_entry_0"), true);
}

/*
#[test]
fn test_good_entry_1() {
    assert_eq!(verify_testcase("good_entry_1"), true);
}
*/

#[test]
fn test_good_entry_2() {
    assert_eq!(verify_testcase("good_entry_2"), true);
}

#[test]
fn test_good_entry_3() {
    assert_eq!(verify_testcase("good_entry_3"), true);
}
