pub mod sandboxer {
    use inkwell::module::Module;
    use inkwell::values::FunctionValue;
    use inkwell::values::PointerValue;
    use inkwell::values::IntValue;
    use inkwell::values::BasicValueEnum;
    use inkwell::values::InstructionOpcode::{Call, Load, Store};

    /// Removes a function call in the body of an LLVM IR function
    pub fn remove_function_call(module: &Module, caller_name: &str, callee_name: &str) {
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

    /// Checks if a given PointerValue is contained within a vector of protected PointerValues.
    fn _is_address_protected(protected_ptrs: &[(PointerValue, IntValue)], ptr: &PointerValue, size: u64) -> bool {

        for &(protected_ptr, protected_size) in protected_ptrs {

            let Some(protected_size_as_u64) = protected_size.get_zero_extended_constant() else { todo!(); };

            println!("protected_ptr: {:?}\n", protected_ptr);
            println!("ptr: {:?}\n", ptr);
            if protected_ptr.eq(ptr) && protected_size_as_u64 >= size {
                return true; // Found a match, return true
            }

        }

        false // No match found, return false
    }

    /// Statically verifies that the memory accesses of a function are safe
    /// Looks for `utx1` functions to protect addresses and checks `load` and `store`.
    pub fn verify(function: FunctionValue) -> bool {

        // Keeps track of protected memory addresses
        // Pointer and size
        let mut protected_ptrs: Vec<(PointerValue, IntValue)> = Vec::new();

        // Iterate over the basic blocks in the function
        for basic_block in function.get_basic_blocks() {

            // Iterate over the instructions in the basic block
            let mut instr_iter = basic_block.get_instructions();
            while let Some(instr) = instr_iter.next() {

                // Call
                if instr.get_opcode() == Call {

                    // Check if it is the call to `utx1`
                    if instr.to_string().contains("utx1") {         // Not sure if this is safe

                        // Extract pointer value and size to protect
                        let (BasicValueEnum::PointerValue(ptr), BasicValueEnum::IntValue(size)) = (
                            instr.get_operand(0).unwrap().unwrap_left(),
                            instr.get_operand(1).unwrap().unwrap_left()
                            ) else { todo!(); };

                        protected_ptrs.push((ptr, size));
                    }

                }

                // Load
                if instr.get_opcode() == Load {

                    let Ok(alignment) = instr.get_alignment() else { todo!(); };

                    let BasicValueEnum::PointerValue(ptr) = instr.get_operand(0).unwrap().unwrap_left() else { todo!(); };

                    if !_is_address_protected(&protected_ptrs, &ptr, alignment as u64) {
                        println!("{:?}", protected_ptrs);
                        return false;
                    }

                }

                // Store
                if instr.get_opcode() == Store {

                    let Ok(alignment) = instr.get_alignment() else { todo!(); };

                    let BasicValueEnum::PointerValue(ptr) = instr.get_operand(1).unwrap().unwrap_left() else { todo!(); };

                    if !_is_address_protected(&protected_ptrs, &ptr, alignment as u64) {
                        println!("{:?}", protected_ptrs);
                        return false;
                    }
                }
            }
        }

        println!("{:?}", protected_ptrs);
        // Return true if the check passes
        true
    }
}
