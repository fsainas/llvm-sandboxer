use std::env;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use llvm_sandboxer::runtime;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the expected number of arguments are provided
    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <function_to_check>", args[0]);
        std::process::exit(1);
    }

    // Get the file path
    let file_path = &args[1];
    let function_name = &args[2];
    let path = Path::new(file_path);

    // Create context
    let context = Context::create();

    // Parse LLVM
    let module = Module::parse_bitcode_from_path(path, &context).unwrap();

    if let Err(err) = runtime::instrument(function_name, &context, &module) {
        println!("Error occurred: {:?}", err);
    } else {
        println!("Instrumentation completed successfully");
    }
    
    module.print_to_stderr();
}
