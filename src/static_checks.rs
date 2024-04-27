use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::context::Context;
use inkwell::values::PointerValue;
use inkwell::values::BasicValueEnum::{PointerValue as PV, IntValue as IV};
use inkwell::values::InstructionOpcode::{Call, Load, Store};
use inkwell::IntPredicate::SLE;

/// Removes a function call in the body of an LLVM IR function
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

/// Checks if a given PointerValue is contained within a vector of protected PointerValues.
fn _is_address_protected(
    protected_mem: &(Option<PointerValue>, Option<u64>), 
    ptr: PointerValue, 
    offset: u64) -> bool {

    let (Some(protected_ptr), Some(protected_offset)) = *protected_mem else { return false; };

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

    false // No match found, return false
}

/// Statically verifies that the memory accesses of a function are safe
/// Looks for `utx1` functions to protect addresses and checks `load` and `store`.
pub fn verify(function: FunctionValue) -> bool {

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

                    if !_is_address_protected(&protected_mem, ptr, alignment as u64) {
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

                    if !_is_address_protected(&protected_mem, ptr, alignment as u64) {
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
