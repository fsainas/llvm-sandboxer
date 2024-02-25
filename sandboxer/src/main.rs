use std::env;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;

fn remove_function_call(module: &Module, caller_name: &str, callee_name: &str) {
    let function = module.get_function(caller_name).unwrap();
    
    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {
        // Iterate over the instructions in the basic block
        let mut instr_iter = basic_block.get_instructions();
        while let Some(instr) = instr_iter.next() {
            // Check if the instruction is a call instruction
            if instr.get_opcode() == inkwell::values::InstructionOpcode::Call {
                // Check if it is the call to remove
                if instr.to_string().contains(callee_name) {
                    instr.erase_from_basic_block();
                }
            }
        }
    }
}


fn main() {
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
    remove_function_call(&module, "main", "utx");

    // Print modified LLVM module
    println!("{}", module.print_to_string().to_string());
}
