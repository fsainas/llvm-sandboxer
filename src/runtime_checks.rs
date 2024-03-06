//! Adds runtime safeguards to llvm microtransactions.

use inkwell::values::InstructionOpcode::{Load, Store};
use inkwell::values::FunctionValue;

/// Given a LLVM function adds runtime memory checks
pub fn instrument(function: FunctionValue) -> FunctionValue {

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        let instructions = basic_block.get_instructions();
        for instr in instructions {

            // Load
            if instr.get_opcode() == Load {

            }

            // Store
            if instr.get_opcode() == Store {

            }

        }

    }

    function

}
