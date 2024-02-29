pub mod sandboxer {
    use inkwell::module::Module;
    use inkwell::values::FunctionValue;
    use inkwell::values::PointerValue;
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
    fn _is_address_protected(protected_ptrs: &Vec<PointerValue>, ptr: PointerValue) -> bool {
        return protected_ptrs.
            iter().any(|protected_ptr| protected_ptr.eq(&ptr))
    }

    /// Statically verifies that the memory accesses of a function are safe
    /// Looks for `utx1` functions to protect addresses and checks `load` and `store`.
    pub fn verify(function: FunctionValue) -> bool {

        // Keeps track of the protected memory addresses
        let mut protected_ptrs: Vec<PointerValue> = Vec::new();

        // Iterate over the basic blocks in the function
        for basic_block in function.get_basic_blocks() {

            // Iterate over the instructions in the basic block
            let mut instr_iter = basic_block.get_instructions();
            while let Some(instr) = instr_iter.next() {

                // Check if the instruction is a call
                if instr.get_opcode() == Call {

                    // Check if it is the call to `utx1`
                    if instr.to_string().contains("utx1") {         // Not sure if this is safe

                        // Exptract pointer value and put it in protected memory
                        match instr.get_operand(0).unwrap().unwrap_left() {     // Clean this
                            BasicValueEnum::PointerValue(ptr) => {
                                protected_ptrs.push(ptr);
                            }
                            _ => { todo!(); }
                        }
                    }
                }

                // Check if the instruction is a Load
                if instr.get_opcode() == Load {

                    // Here we need to extract the address accessed in the shared_array
                    match instr.get_operand(0).unwrap().unwrap_left() {                     // Clean
                        BasicValueEnum::PointerValue(ptr) => {
                            // If there is no value in the protected_ptrs return false
                            if !_is_address_protected(&protected_ptrs, ptr) {
                                return false;
                            }
                        }
                        _ => { todo!(); }
                    }
                }

                if instr.get_opcode() == Store {

                    match instr.get_operand(1).unwrap().unwrap_left() {
                        BasicValueEnum::PointerValue(ptr) => {
                            // If there is no value in the protected_ptrs return false
                            if !_is_address_protected(&protected_ptrs, ptr) {
                                return false;
                            }
                        }
                        _ => { todo!(); }
                    }
                }
            }
        }

        true
    }

}
