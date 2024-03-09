//! Adds runtime safeguards to llvm microtransactions.

use inkwell::values::InstructionOpcode::{Load, Store};
use inkwell::context::Context;
use inkwell::basic_block::BasicBlock;
//use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::values::InstructionValue;
use inkwell::attributes::Attribute;
use inkwell::attributes::AttributeLoc;

// Assume `context` and `module` are already created
 /*
fn _add_abort_function<'ctx>(context: /*&'ctx*/ &Context, module: &Module/*<'ctx>*/) /*-> FunctionValue<'ctx>*/ {
    // Define the type of the abort function: fn() -> void
    let abort_type = context.void_type().fn_type(&[], false);

    // Create the abort function in the module
    let abort_func = module.add_function("abort", abort_type, None);

    // Define the function body (empty for an abort function)
    //let basic_block = context.append_basic_block(&abort_func, "entry");
    //let builder = context.create_builder();
    //builder.position_at_end(&basic_block);
    //builder.build_return(None);

    //abort_func
}
*/

/// Moves an instruction `instr` and the following ones to a new block `to_block`
fn _move_instructions(context: &Context, instr: InstructionValue, to_block: &BasicBlock) {

    let builder = context.create_builder();
    builder.position_at_end(*to_block);

    let mut current_instr = instr;
    while let Some(next_instr) = current_instr.get_next_instruction() {
        current_instr.remove_from_basic_block();
        builder.insert_instruction(&current_instr, None);
        current_instr = next_instr;
    }

    // Last instruction
    current_instr.remove_from_basic_block();
    builder.insert_instruction(&current_instr, None);

}

/// Given a LLVM function adds runtime memory checks
pub fn instrument<'a, 'b>(
    function: FunctionValue, 
    context: &'a Context, 
    module: &'b Module<'a>) /*-> FunctionValue*/ {

    // Define the type of the abort function: fn() -> void
    let abort_type = context.void_type().fn_type(&[], false);

    // Create the abort function in the module
    let abort_func = module.add_function("abort", abort_type, None);

    // ***** Append abort block ***** //
    let abort_basic_block = context.append_basic_block(function, "abort");

    // Create builder and position at the end of the abort basic block
    let abort_builder = context.create_builder();
    abort_builder.position_at_end(abort_basic_block);

    // Call abort function with noreturn and nounwind attrs
    let call_abort_instr = abort_builder.build_call(abort_func, &[], "abort");

    /* Add noreturn and nounwind attributes
    call_abort_instr
        .expect("boh")
        .add_attribute(AttributeLoc::Return, Attribute::get(context, "noreturn").unwrap());
    call_abort_instr
        .expect("boh")
        .add_attribute(AttributeLoc::Return, context.create_string_attribute("nounwind","nounwind"));
        */

    abort_builder.build_unreachable();

    let mut load_counter = 0;
    let mut store_counter = 0;

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        let instructions = basic_block.get_instructions();
        for instr in instructions {

            match instr.get_opcode() {

                Load => {

                    // Create the block to store the rest of the code
                    let block_name = format!("load{}", load_counter);
                    let continue_block = context.insert_basic_block_after(basic_block, &block_name);
                    load_counter += 1;

                    // Create a new builder
                    let block_builder = context.create_builder();

                    // Position the builder before the Load instruction
                    block_builder.position_before(&instr);

                    // TODO: Tmp value to test, remove with the actual check
                    let cmp = context.i64_type().const_int(1, false);

                    // Create the instruction that evaluates cmp and chose to abort or continue
                    block_builder.build_conditional_branch(cmp, abort_basic_block, continue_block);

                    // Move instructions to the continue_block
                    _move_instructions(&context, instr, &continue_block);

                }

                Store => {

                    // Create the block to store the rest of the code
                    let block_name = format!("store{}", store_counter);
                    let continue_block = context.insert_basic_block_after(basic_block, &block_name);
                    store_counter += 1;

                    // Create a new builder
                    let block_builder = context.create_builder();

                    // Position the builder before the Load instruction
                    block_builder.position_before(&instr);

                    // TODO: Tmp value to test, remove with the actual check
                    let one_int_val = context.i64_type().const_int(1, false);

                    // Create the instruction that evaluates cmp and chose to abort or continue
                    block_builder.build_conditional_branch(one_int_val, abort_basic_block, continue_block);

                    // Move instructions to the continue_block
                    _move_instructions(&context, instr, &continue_block);

                }

                _ => {}

            }

        }

    }

    //function.clone()

}
