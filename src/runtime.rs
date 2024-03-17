//! Adds runtime safeguards to llvm microtransactions.

// External crates
use either::Left;

// Inkwell imports
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, GlobalValue, InstructionValue};
use inkwell::values::{IntValue, PointerValue};
use inkwell::IntPredicate::*;

// Instruction opcodes
use inkwell::values::InstructionOpcode::{Call, Load, Store};

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

fn _build_check(
    context: &Context,
    builder: Builder,
    protected_ptr_val: PointerValue,
    accessed_ptr_val: PointerValue,
    protected_offset: GlobalValue,
    alignment_as_int_value: IntValue,
    abort_block: BasicBlock,
    continue_block: BasicBlock,
    block_name: &str
    ) -> Result<(), String> {
    let i64_type = context.i64_type();
    let pointer_type = context.i8_type().ptr_type(inkwell::AddressSpace::default());

    // Load protected offset value
    let protected_offset_val = match builder.build_load(
        i64_type,
        protected_offset.as_pointer_value(),
        &format!("protected_offset_{}", block_name),
        ) {
        Ok(value) => value.into_int_value(),
        Err(_) => return Err("Failed to load value for 'protected_offset_val'".to_string()),
    };


    // Compare accessed pointer value with protected pointer value
    let accessed_lt_protected = builder.build_int_compare(
        SLT, 
        accessed_ptr_val, 
        protected_ptr_val, 
        &format!("accessed_lt_protected_{}", block_name)
        ).map_err(|e| format!("Failed to build integer comparison for 'accessed_ptr_val' < 'protected_ptr_val': {:?}", e))?;

    unsafe {

        // Perform pointer arithmetic for last protected pointer value
        let last_protected_ptr_val = builder.build_gep(
            pointer_type, 
            protected_ptr_val, 
            &[protected_offset_val], 
            &format!("last_protected_ptr_{}", block_name)
            ).map_err(|e| format!("Failed to build pointer arithmetic for 'last_protected_ptr_val': {:?}", e))?;

        // Perform pointer arithmetic for last accessed pointer value
        let last_accessed_ptr_val = builder.build_gep(
            pointer_type, 
            accessed_ptr_val, 
            &[alignment_as_int_value], 
            &format!("last_accessed_ptr_{}", block_name)
            ).map_err(|e| format!("Failed to build pointer arithmetic for 'last_accessed_ptr_val': {:?}", e))?;

        // Compare last accessed pointer value with last protected pointer value
        let last_acc_gt_last_prot = builder.build_int_compare(
            SGT, 
            last_accessed_ptr_val, 
            last_protected_ptr_val, 
            &format!("last_acc_gt_last_prot_{}", block_name)
            ).map_err(|e| format!("Failed to build integer comparison for 'last_accessed_ptr_val' > 'last_protected_ptr_val': {:?}", e))?;

        // Build logical OR operation for checks
        let check = builder.build_or(
            accessed_lt_protected, 
            last_acc_gt_last_prot, 
            &format!("check_{}", block_name)
            ).map_err(|e| format!("Failed to build logical OR operation for 'accessed_lt_protected' || 'last_acc_gt_last_prot': {:?}", e))?;

        // Create the instruction that evaluates comparison and chooses to abort or continue
        builder.build_conditional_branch(check, abort_block, continue_block)
            .map_err(|e| format!("Failed to build conditional branch: {:?}", e))?;

    }

    Ok(())
}

/// Given a LLVM function adds runtime memory checks
pub fn instrument<'a>(
    function_name: &str, 
    context: &'a Context, 
    module: &Module<'a>) -> Result<(), String> {

    // Retrive function value
    let function = module.get_function(function_name).unwrap();

    // Define the type of the abort function: fn() -> void
    let abort_type = context.void_type().fn_type(&[], false);

    // Create the abort function in the module
    let abort_func = module.add_function("abort", abort_type, None);

    // ***** Append abort block ***** //
    let abort_block = context.append_basic_block(function, "abort");

    // Create builder and position at the end of the abort basic block
    let abort_builder = context.create_builder();
    abort_builder.position_at_end(abort_block);

    // Call abort function with noreturn and nounwind attrs
    let _ = abort_builder.build_call(abort_func, &[], "abort");

    /* Add noreturn and nounwind attributes
    call_abort_instr
        .expect("boh")
        .add_attribute(AttributeLoc::Return, Attribute::get(context, "noreturn").unwrap());
    call_abort_instr
        .expect("boh")
        .add_attribute(AttributeLoc::Return, context.create_string_attribute("nounwind","nounwind"));
        */

    let _ = abort_builder.build_unreachable();

    /***** Create a global variable for storing the current protected pointer *****/
    // Pointer type for protected pointer
    let pointer_type = context.i8_type().ptr_type(inkwell::AddressSpace::default());
    // Type of the offset
    let i64_type = context.i64_type();

    // Add globals
    let protected_ptr = module.add_global(pointer_type, None, "protected_ptr");
    let protected_offset = module.add_global(i64_type, None, "protected_offset");

    // Initialize globals
    let null_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
    protected_ptr.set_initializer(&null_ptr);

    let zero_offset = i64_type.const_int(0, false);
    protected_offset.set_initializer(&zero_offset);

    // counts the number of load and store instructions, to build labels later
    let mut load_counter = 0;
    let mut store_counter = 0;

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        // Iterate over the instructions in the basic block
        let instructions = basic_block.get_instructions();
        for instr in instructions {

            match instr.get_opcode() {

                Call => {
                    // Check if it is the call to `utx1`
                    if instr.to_string().contains("utx1") {         // Not sure if this is safe

                        // Extract pointer value and offset to protect
                        let (ptr, offset) = match (instr.get_operand(0), instr.get_operand(1)) {
                            (Some(Left(BasicValueEnum::PointerValue(ptr))), Some(Left(BasicValueEnum::IntValue(offset)))) => (ptr, offset),
                            _ => return Err("Failed to extract pointer value and offset".to_string()),
                        };

                        // Create a new builder and position it before the instruction
                        let builder = context.create_builder();
                        builder.position_before(&instr);

                        // Store the pointer value and offset to protect
                        builder.build_store(protected_ptr.as_pointer_value(), ptr)
                            .map_err(|e| format!("Failed to store protected pointer value: {:?}", e))?;
                        builder.build_store(protected_offset.as_pointer_value(), offset)
                            .map_err(|e| format!("Failed to store protected offset value: {:?}", e))?;


                        // remove utx1 call
                        instr.erase_from_basic_block();

                    }

                }

                Load => {

                    // Create the block to store the rest of the code
                    let block_name = format!("load{}", load_counter);
                    load_counter += 1;
                    let continue_block = context.insert_basic_block_after(basic_block, &block_name);

                    // Create a new builder and position it before the Load instruction
                    let builder = context.create_builder();
                    builder.position_before(&instr);

                    // Extract pointer values and alignment from the instruction
                    let protected_ptr_val = protected_ptr.as_pointer_value();
                    let accessed_ptr_val = match instr.get_operand(0).unwrap().unwrap_left() {
                        BasicValueEnum::PointerValue(ptr) => ptr,
                        _ => return Err("Failed to extract accessed pointer value.".to_string())
                    };

                    let alignment_as_int_value = i64_type.const_int(
                        instr.get_alignment().expect("Failed to get alignment") as u64, false
                        );

                    _build_check(
                        context, 
                        builder, 
                        protected_ptr_val, 
                        accessed_ptr_val, 
                        protected_offset,
                        alignment_as_int_value,
                        abort_block,
                        continue_block,
                        &block_name)?;

                    // Move instructions to the continue_block
                    _move_instructions(context, instr, &continue_block);

                }

                Store => {

                    // Create the block to store the rest of the code
                    let block_name = format!("store{}", store_counter);
                    let continue_block = context.insert_basic_block_after(basic_block, &block_name);
                    store_counter += 1;

                    // Create a new builder and position it before the Load instruction
                    let builder = context.create_builder();
                    builder.position_before(&instr);

                    // Extract pointer values and alignment from the instruction
                    let protected_ptr_val = protected_ptr.as_pointer_value();
                    let accessed_ptr_val = match instr.get_operand(1).unwrap().unwrap_left() {
                        BasicValueEnum::PointerValue(ptr) => ptr,
                        _ => return Err("Failed to extract accessed pointer value.".to_string())
                    };

                    let alignment_as_int_value = i64_type.const_int(
                        instr.get_alignment().expect("Failed to get alignment") as u64, false
                        );

                    _build_check(
                        context, 
                        builder, 
                        protected_ptr_val, 
                        accessed_ptr_val, 
                        protected_offset,
                        alignment_as_int_value,
                        abort_block,
                        continue_block,
                        &block_name)?;

                    // Move instructions to the continue_block
                    _move_instructions(context, instr, &continue_block);

                }

                _ => {}

            }

        }

    }

    Ok(())
}
