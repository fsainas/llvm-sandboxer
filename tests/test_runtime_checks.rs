use llvm_sandboxer::runtime_checks;

use std::path::Path;
use std::process::Command;
use inkwell::module::Module;
use inkwell::context::Context;
use inkwell::values::FunctionValue;

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
    
    // Retrive function value
    let function = module.get_function(testcase_name).unwrap();

    runtime_checks::instrument(function, &context, &module);
    
    println!("{}", module.print_to_string().to_string());

}

#[test]
fn test_instrument_bad_entry_0() {
    //assert_eq!(instrument_testcase("bad_entry_0"), true);
    //println!("{:?}", instrument_testcase("bad_entry_0"));
    instrument_testcase("bad_entry_0");
    assert_eq!(false, true);
}
