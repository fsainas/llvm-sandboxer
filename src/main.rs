use std::env;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use llvm_sandboxer::sandboxer;

fn main() {
    /* Remove `utx` code
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the expected number of arguments are provided
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    // Get the file path
    let file_path = &args[1];
    let path = Path::new(file_path);

    // Create context
    let context = Context::create();

    // Parse LLVM
    let module = Module::parse_bitcode_from_path(&path, &context).unwrap();

    // Process LLVM
    sandboxer::remove_function_call(&module, "main", "utx");

    // Print modified LLVM module
    println!("{}", module.print_to_string().to_string());
    */

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

    // Process LLVM
    let function = module.get_function(function_name).unwrap();
    let result = sandboxer::verify(function);
    println!("{}", result);
}
