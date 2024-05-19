use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use std::path::Path;
use inkwell::module::Module;
use inkwell::context::Context;
use llvm_sandboxer::runtime;

/// Compile test cases's C sources.
fn compile_c_files() {
    let no_utx_dir_path = Path::new("benches/c_files/no_utx/");
    let utx_dir_path = Path::new("benches/c_files/utx/");

    // Compile the C source file
    Command::new("make")
        .args(["-C", &no_utx_dir_path.to_string_lossy()])
        .output()
        .expect("Failed to compile C source file.");

    // Compile the C source file
    Command::new("make")
        .args(["-C", &utx_dir_path.to_string_lossy()])
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
fn instrument_test_case(test_case_name: &str) -> String {

    // Get testcase bitcode path
    let bitcode_path = format!("target/bench/utx/{}.bc", test_case_name);
    let bitcode_path = Path::new(&bitcode_path);
    
    // Compile if it not exists
    if !bitcode_path.exists() {
        compile_c_files();
    }

    // Parse bitcode
    let context = Context::create();
    let module = Module::parse_bitcode_from_path(&bitcode_path, &context).unwrap();

    match runtime::instrument(test_case_name, &context, &module, true) {
        Ok(()) => println!("Instrumentation completed successfully"),
        Err(err) => println!("Error occurred: {:?}", err)
    }

    // Save to file
    let filepath = format!("target/bench/instrumented/{}_instrumented.ll", test_case_name);
    let _ = module.print_to_file(filepath.clone());

    return filepath
}

fn benchmark_0(c: &mut Criterion) {

    let test_case_name = "benchmark_0";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("benchmark_0", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn benchmark_0_instrumented(c: &mut Criterion) {
    let test_case_name = "benchmark_0";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_benchmark_0", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn benchmark_1(c: &mut Criterion) {

    let test_case_name = "benchmark_1";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("benchmark_1", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn benchmark_1_instrumented(c: &mut Criterion) {
    let test_case_name = "benchmark_1";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_benchmark_1", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

criterion_group!(benches, benchmark_0, benchmark_0_instrumented, benchmark_1, benchmark_1_instrumented);
criterion_main!(benches);
