use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::context::Context;
use inkwell::values::PointerValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::InstructionOpcode::{Call, Load, Store};
use inkwell::IntPredicate::SLE;

/// Removes a function call in the body of an LLVM IR function
pub fn remove_function_call(module: &Module, caller_name: &str, callee_name: &str) {
    let function = module.get_function(caller_name).unwrap();

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        let instr_iter = basic_block.get_instructions();
        for instr in instr_iter {

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

/// Checks if a given PointerValue is contained within a vector of protected PointerValues.
fn _is_address_protected(
    protected_ptrs: &[(PointerValue, u64)], 
    ptr: PointerValue, 
    offset: u64) -> bool {

    for &(protected_ptr, protected_offset) in protected_ptrs {

        // Protected pointer and pointer accessed are the same
        if protected_ptr == ptr && protected_offset >= offset {
            return true;
        } else {
            let context = Context::create();
            let i64_type = context.i64_type();

            unsafe {
                // Cast offset to IntValue
                let protected_offset_as_int_value = i64_type.const_int(protected_offset, false);
                // Compute the last protected address (protected_ptr + offset)
                let last_protected_ptr = protected_ptr.const_gep(i64_type, &[protected_offset_as_int_value]); 


                // Cast protected_ptr to IntValue
                let protected_ptr_as_int_value = protected_ptr.const_to_int(i64_type);
                let last_protected_ptr_as_int_value = last_protected_ptr.const_to_int(i64_type);
                let ptr_as_int_value = ptr.const_to_int(i64_type);
                println!("PTR: {:?}", protected_ptr_as_int_value);
                println!("LAST PTR: {:?}", last_protected_ptr_as_int_value);
                // Check if ptr points inside the range [protected_ptr, last_protected_pointer]
                let res = protected_ptr_as_int_value.const_int_compare(SLE, ptr_as_int_value).get_sign_extended_constant();
                println!("RES: {:?}", res);

                // TODO
            }

        }

    }

    false // No match found, return false
}

/// Statically verifies that the memory accesses of a function are safe
/// Looks for `utx1` functions to protect addresses and checks `load` and `store`.
pub fn verify(function: FunctionValue) -> bool {

    // Keeps track of protected memory addresses
    // (pointer, offset)
    let mut protected_ptrs: Vec<(PointerValue, u64)> = Vec::new();

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        let instr_iter = basic_block.get_instructions();
        for instr in instr_iter {

            // Call
            if instr.get_opcode() == Call {

                // Check if it is the call to `utx1`
                if instr.to_string().contains("utx1") {         // Not sure if this is safe

                    // Extract pointer value and offset to protect
                    let (BasicValueEnum::PointerValue(ptr), BasicValueEnum::IntValue(offset)) = (
                        instr.get_operand(0).unwrap().unwrap_left(),
                        instr.get_operand(1).unwrap().unwrap_left()
                        ) else { todo!(); };

                    let Some(offset_as_u64) = offset.get_zero_extended_constant() else { todo!() };
                    protected_ptrs.push((ptr, offset_as_u64));

                }

            }

            // Load
            if instr.get_opcode() == Load {

                let Ok(alignment) = instr.get_alignment() else { todo!(); };

                let BasicValueEnum::PointerValue(ptr) = instr.get_operand(0).unwrap().unwrap_left() else { todo!(); };

                if !_is_address_protected(&protected_ptrs, ptr, alignment as u64) {
                    return false;
                }

            }

            // Store
            if instr.get_opcode() == Store {

                let Ok(alignment) = instr.get_alignment() else { todo!(); };

                let BasicValueEnum::PointerValue(ptr) = instr.get_operand(1).unwrap().unwrap_left() else { todo!(); };

                if !_is_address_protected(&protected_ptrs, ptr, alignment as u64) {
                    return false;
                }

            }

        }

    }

    // Return true if the check passes
    true
}
