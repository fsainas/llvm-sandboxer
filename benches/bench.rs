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

fn vec_sum(c: &mut Criterion) {

    let test_case_name = "vec_sum";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("vec_sum", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn vec_sum_instrumented(c: &mut Criterion) {
    let test_case_name = "vec_sum";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_vec_sum", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn bubble_sort(c: &mut Criterion) {

    let test_case_name = "bubble_sort";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("bubble_sort", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn bubble_sort_instrumented(c: &mut Criterion) {
    let test_case_name = "bubble_sort";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_bubble_sort", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn simple_store(c: &mut Criterion) {

    let test_case_name = "simple_store";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("simple_store", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn simple_store_instrumented(c: &mut Criterion) {
    let test_case_name = "simple_store";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_simple_store", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn matrix_mul(c: &mut Criterion) {

    let test_case_name = "matrix_mul";

    let exec_path = format!("target/bench/no_utx/{}.o", test_case_name);
    let exec_path = Path::new(&exec_path);

    // Compile if it not exists
    if !exec_path.exists() {
        compile_c_files();
    }

    c.bench_function("matrix_mul", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

fn matrix_mul_instrumented(c: &mut Criterion) {
    let test_case_name = "matrix_mul";

    let ll_filepath = instrument_test_case(test_case_name);
    let exec_path = compile_ll_to_exec(&ll_filepath);
    let exec_path = Path::new(&exec_path);

    c.bench_function("instrumented_matrix_mul", |b| {
        b.iter(|| {
            let _ = Command::new(exec_path)
                .output()
                .expect(&format!("Cannot execute {}.", exec_path.display()));
        });
    });
}

criterion_group!(benches, /*vec_sum, vec_sum_instrumented, bubble_sort,
bubble_sort_instrumented, simple_store, simple_store_instrumented,*/ matrix_mul, matrix_mul_instrumented);
criterion_main!(benches);
