//! Provides static checks for protected memory addresses.
//!
//! This module offers functions to verify and enforce memory access safety
//! within LLVM IR functions.  It includes methods to remove utxs function calls
//! and verify memory accesses against protected addresses.
//!
//! # Example
//!
//! ```rust
//! use inkwell::context::Context;
//! use inkwell::module::Module;
//! use inkwell::values::{FunctionValue, PointerValue};
//! use llvm_sandboxer::static_checks::{remove_function_call, verify};
//!
//! fn main() {
//!     // Initialize LLVM context and module
//!     let context = Context::create();
//!     let module = context.create_module("example");
//!
//!     // Define a sample function and add it to the module
//!     let function = module.add_function(
//!         "example_function",
//!         context.i64_type().fn_type(&[], false),
//!         None,
//!     );
//!
//!     // Perform static checks on the module
//!     let checks_passed = verify(module, function);
//!     if checks_passed {
//!         println!("Memory access checks passed.");
//!     } else {
//!         println!("Memory access checks failed.");
//!     }
//! }
//! ```

use inkwell::module::Module;
use inkwell::values::{AnyValue, FunctionValue};
use inkwell::values::PointerValue;
use inkwell::values::BasicValueEnum::{PointerValue as PV, IntValue as IV};
use inkwell::values::InstructionOpcode::{Call, Load, Store};
use regex::Regex;

/// Removes a specific function call from the body of an LLVM IR function.
///
/// # Arguments
///
/// * `module` - A reference to the LLVM module containing the function.
/// * `caller_name` - The name of the function from which to remove the call.
/// * `callee_name` - The name of the function call to remove.
pub fn remove_function_call(module: &Module, caller_name: &str, callee_name: &str) {

    let function = module.get_function(caller_name).unwrap();

    // Iterate over the basic blocks in the function
    for bb in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        for instr in bb.get_instructions() {

            // Check if the instruction is a call instruction
            if instr.get_opcode() == Call {

                // Check if it is the call to remove
                if instr.to_string().contains(callee_name) {

                    instr.erase_from_basic_block();

                }

            }

        }

    }

}

fn _get_type_as_str_size(type_as_str: &str) -> i64 {

    match type_as_str {
        "i64" => 8,
        other => panic!("Expected type, found {}", other)
    }

}

fn _parse_gep(gep: PointerValue) -> Vec<(String, String)> {

    let gep_as_llvmstring = gep.print_to_string();
    let gep_as_str: &str = gep_as_llvmstring.to_str().expect("Failed to convert LLVMString to &str.");

    // (type, value)
    let mut operands: Vec<(String, String)> = Vec::new();

    // Get type
    let gep_type_pattern: Regex = Regex::new(r"(\w)+ x (\w)+").unwrap();

    let capture = gep_type_pattern.captures(gep_as_str).unwrap();

    let binding = capture[0].to_string().replace(" x ", " ");
    let array_type_and_size_as_string: Vec<_> = binding.split_whitespace().collect();

    let array_size= array_type_and_size_as_string[0].to_string();
    let array_type= array_type_and_size_as_string[1].to_string();

    operands.push((array_size, array_type));

    // Pattern to match GEP operands
    let gep_operands_pattern: Regex = Regex::new(r", (\w)+ @?(\w)+").unwrap();

    for capture in gep_operands_pattern.captures_iter(gep_as_str) {

        let binding: String = capture[0].to_string().replace(",", "");
        let parts: Vec<_> = binding.split_whitespace().collect();

        operands.push((parts[0].to_string(), parts[1].to_string()));
    }

    operands

}

/// Checks if a given pointer value is in a protected range.
///
/// This function compares the provided pointer value against a protected memory address and offset.
/// If the pointer value matches the protected address and the offset is within the protected range,
/// the function returns true, indicating that the memory access is protected.
///
/// # Arguments
///
/// * `module` - The LLVM module containing the global variables.
/// * `protected_mem` - A tuple containing the protected memory address and offset.
/// * `ptr` - The pointer value to check for protection.
/// * `alignment` - The alignment associated with the pointer value.
///
/// # Returns
///
/// Returns true if the pointer value is protected, false otherwise.
fn _is_address_protected(
    module: Module,
    protected_mem: &(Option<PointerValue>, Option<u64>), 
    ptr: PointerValue, 
    alignment: u64) -> bool {

    let (Some(protected_ptr), Some(protected_offset)) = *protected_mem else { return false; };

    // Protected pointer and pointer accessed are the same
    if protected_ptr == ptr && protected_offset >= alignment {

        return true;

    } 

    // If the pointer is not constant, the value cannot be computed statically
    if !ptr.is_const() { return false }

    // Parse get element pointer and get base pointer, offset and type size
    let gep_operands: Vec<(String, String)> = _parse_gep(ptr);

    let size: i64 = _get_type_as_str_size(&gep_operands[0].1);

    // The first operand is the base pointer
    // Remove the '@'
    let base_ptr_as_string: String = gep_operands[1].1.replace("@", "");

    let first_index: i64 = gep_operands[2].1.parse().unwrap();

    // Overflow
    if first_index > 0 { return  false }

    let offset: i64 = gep_operands[3].1.parse().unwrap();

    // Get base pointer from base pointer string
    let base_ptr: PointerValue = match module.get_global(&base_ptr_as_string.as_str()) {
        Some(global_val) => global_val.as_pointer_value(),
        None => todo!()
    };

    // Check if the base pointer is protected and if the accessed pointer is inside the protected range
    if protected_ptr.print_to_string() == base_ptr.print_to_string() && (protected_offset as i64) >= offset * size {

        return true;

    }

    false // No match found, return false
}

/// Statically verifies that memory accesses within a function are safe.
///
/// This function iterates over the instructions in a function's basic blocks,
/// checking for load and store operations. It ensures that memory accesses
/// do not violate protected memory addresses.
///
/// # Arguments
///
/// * `module` - The LLVM module containing the function.
/// * `function` - The LLVM IR function to verify.
///
/// # Returns
///
/// Returns `true` if memory access checks pass, `false` otherwise.
pub fn verify(module: Module, function: FunctionValue) -> bool {

    // Keeps track of protected memory addresses
    // (pointer, offset)
    let mut protected_mem: (Option<PointerValue>, Option<u64>) = (None, None);

    // Iterate over the basic blocks in the function
    for bb in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        for instr in bb.get_instructions() {

            match instr.get_opcode() {

                Call => {

                    // Check if it is the call to `utx1`
                    if instr.to_string().contains("utx1") {         // Not sure if this is safe

                        // Extract pointer value and offset to protect
                        let ptr: PointerValue = match instr.get_operand(0).unwrap().unwrap_left() {
                            PV(ptr) => ptr,
                            other => panic!("Expected PointerValue, found {:?}", other),
                        };

                        let offset = match instr.get_operand(1).unwrap().unwrap_left() {
                            IV(offset) => offset,
                            other => panic!("Expected IntValue, found {:?}", other),
                        };

                        let Some(offset_as_u64) = offset.get_zero_extended_constant() else { todo!() };
                        protected_mem = (Some(ptr), Some(offset_as_u64));

                    }
                
                }

                Load => {

                    let alignment: u32 = instr.get_alignment()
                    .expect(&format!("Failed to get the alignment of instruction {:?}", instr));

                    let ptr: PointerValue = match instr.get_operand(0).unwrap().unwrap_left() { 
                        PV(ptr) => ptr,
                        other => panic!("Expected PointerValue, found {:?}", other)
                    };

                    if !_is_address_protected(module.clone(), &protected_mem, ptr, alignment as u64) {
                        return false;
                    }

                }

                Store => {

                    let alignment: u32 = instr.get_alignment()
                    .expect(&format!("Failed to get the alignment of instruction {:?}", instr));

                    let ptr: PointerValue = match instr.get_operand(1).unwrap().unwrap_left() { 
                        PV(ptr) => ptr,
                        other => panic!("Expected PointerValue, found {:?}", other)
                    };

                    if !_is_address_protected(module.clone(), &protected_mem, ptr, alignment as u64) {
                        return false;
                    }

                }

                _ => ()

            }

        }

    }

    // Check passes
    true

}
