//! Adds runtime safeguards to llvm micro-transactions.

use std::str::FromStr;

// External crates
use either::*;

// Inkwell imports
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, GlobalValue, InstructionValue, FunctionValue};
use inkwell::values::{IntValue, PointerValue, PhiValue};
use inkwell::values::BasicValueEnum::PointerValue as PV;
use inkwell::IntPredicate::*;
use inkwell::values::AnyValue;
use inkwell::types::AnyTypeEnum::{ArrayType, FloatType, IntType, PointerType, StructType, VectorType};
use inkwell::types::BasicTypeEnum;

extern crate llvm_sys as llvm;

use regex::Regex;

// Instruction opcodes
use inkwell::values::InstructionOpcode::{Call, Load, Store, Phi, Br, Alloca};

use crate::static_checks;

/// Moves an instruction `instr` and the following ones to a new block `to_block`
fn _move_instructions(context: &Context, instr: &InstructionValue, to_block: &BasicBlock) {

    let builder = context.create_builder();
    builder.position_at_end(*to_block);

    let mut current_instr = *instr;
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
    protected_mem: (GlobalValue, GlobalValue),
    accessed_mem: (PointerValue, IntValue),
    abort_block: &BasicBlock,
    continue_block: BasicBlock,
    block_name: &str
    ) -> Result<(), String> {

    let i64_type = context.i64_type();
    let ptr_type = context.i8_type().ptr_type(inkwell::AddressSpace::default());

    // Unpacking
    let protected_ptr: GlobalValue<'_> = protected_mem.0;
    let protected_offset: GlobalValue<'_> = protected_mem.1;
    let accessed_ptr_val: PointerValue<'_> = accessed_mem.0;
    let alignment_as_int_value: IntValue<'_> = accessed_mem.1;

    // Load pointer value from @protected_ptr
    let protected_ptr_val: PointerValue = match builder.build_load(
        ptr_type,
        protected_ptr.as_pointer_value(),
        &format!("protected_ptr_{}", block_name),) {
        Ok(value) => value.into_pointer_value(),
        Err(_) => return Err("Failed to build load from @protected_val".to_string()),
    };

    // Check if @protected_ptr is null
    let null_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
    let protected_is_null = match builder.build_int_compare(
        EQ, 
        protected_ptr_val, 
        null_ptr,
        &format!("protected_is_null_{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to build check for null protected pointer".to_string())
    };

    // Compare accessed pointer value with protected pointer value
    let accessed_lt_protected = match builder.build_int_compare(
        SLT, 
        accessed_ptr_val, 
        protected_ptr_val, 
        &format!("accessed_lt_protected_{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to build integer comparison for 'accessed_ptr_val' < 'protected_ptr_val'".to_string())
    };

    // Load protected offset value
    let protected_offset_val = match builder.build_load(
        i64_type,
        protected_offset.as_pointer_value(),
        &format!("protected_offset_{}", block_name),
        ) {
        Ok(value) => value.into_int_value(),
        Err(_) => return Err("Failed to load value for 'protected_offset_val'".to_string()),
    };

    // Convert protected pointer to int to compute the last protected pointer
    let protected_ptr_val_as_int = match builder.build_ptr_to_int(
        protected_ptr_val, 
        i64_type, 
        &format!("protected_ptr_as_int{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to convert @protected_ptr to int".to_string()),
    };

    // Compute last protected pointer
    let last_protected_ptr_val = match builder.build_int_add(
        protected_ptr_val_as_int,
        protected_offset_val, 
        &format!("last_protected_ptr_{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to build last_protected_ptr_val calculation.".to_string()),
    };

    // Convert accessed pointer to int to compute the last accessed pointer
    let accessed_ptr_val_as_int = match builder.build_ptr_to_int(
        accessed_ptr_val, 
        i64_type, 
        &format!("accessed_ptr_as_int{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to cast pointer to int".to_string()),
    };

    // Compute last accessed pointer
    let last_accessed_ptr_val_as_int = match builder.build_int_add(
        accessed_ptr_val_as_int,
        alignment_as_int_value, 
        &format!("last_accessed_ptr_as_int{}", block_name)) {
        Ok(value) => value,
        Err(_) => return Err("Failed to build last_protected_ptr_val calculation.".to_string()),
    };

    // Compare last accessed pointer value with last protected pointer value
    let last_acc_gt_last_prot = match builder.build_int_compare(
        SGT, 
        last_accessed_ptr_val_as_int, 
        last_protected_ptr_val,
        &format!("last_acc_gt_last_prot_{}", block_name)) {
            Ok(value) => value,
            Err(_) => return Err("Failed to build integer comparison for 'last_accessed_ptr_val' > 'last_protected_ptr_val'".to_string())
    };

    // Build logical OR operation for checks
    let check_range = match builder.build_or(
        accessed_lt_protected, 
        last_acc_gt_last_prot, 
        &format!("check_range_{}", block_name)) {
            Ok(value) => value,
            Err(_) => return Err("Failed to build logical OR operation for 'accessed_lt_protected' || 'last_acc_gt_last_prot'".to_string())
    };

    // Build logical OR operation for checks
    let check = match builder.build_or(
        protected_is_null, 
        check_range, 
        &format!("check_{}", block_name)) {
            Ok(value) => value,
            Err(_) => return Err("Failed to build logical OR operation for 'accessed_lt_protected' || 'last_acc_gt_last_prot'".to_string())
    };

    // Create the instruction that evaluates comparison and chooses to abort or continue
    match builder.build_conditional_branch(check, *abort_block, continue_block) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed to build conditional branch: {:?}", e))
    }

}


/// Extracts entries of a phi instruction.
fn _get_phi_entries<'a>(instr: &'a InstructionValue<'a>) -> Vec<(BasicValueEnum<'a>, String)> {

    if instr.get_opcode() != Phi {
        panic!("Instruction is not Phi!");
    }

    let instr_as_llvmstring: inkwell::support::LLVMString = instr.print_to_string();
    let instr_as_str: &str = instr_as_llvmstring.to_str().expect("Failed to convert LLVMString to &str.");

    // To store the Phi entries
    let mut entries: Vec<(BasicValueEnum, String)> = Vec::new();

    // Define a regular expression pattern to match phi entries e.g. [ %1, %0 ]
    let entry_pattern: Regex = Regex::new(r" %?(\w)+, %(\w)+").unwrap();

    for (i, capture) in entry_pattern.captures_iter(instr_as_str).enumerate() {

        let binding: String = capture[0].to_string();
        let mut parts: std::str::Split<'_, char> = binding.split(',');

        // If the pattern matches every entry, then we can retrieve the value with the "get_operand()" method
        let value: BasicValueEnum<'_> = instr.get_operand(i as u32)
        .expect("Failed to get value from phi instruction. Reason: Operand not found.")
        .left()
        .expect("Expected BasicValueEnum, found BasicBlock.");

        let bb_name: &str = parts.nth(1)
        .expect("Failed to get the BasicBlock name from capture. Reason: The 'parts' iterator does not contain a second element.");

        entries.push((
            value,
            bb_name.to_string().replace(" %", "")));    // Clean basic block name
    }

    entries

}

/// Updates a phi instruction
fn _update_phi(
    context: &Context, 
    function: &FunctionValue,
    phi_bb: BasicBlock,
    instr: &InstructionValue, 
    previous_bb_name: &str, 
    continue_block: &BasicBlock,
    phi_counter: &mut u32) {

    if instr.get_opcode() != Phi {
        panic!("Instruction is not Phi!");
    }

    // Get entries of Phi instruction
    let entries: Vec<(BasicValueEnum, String)> = _get_phi_entries(instr);

    let mut bb_names: Vec<String> = Vec::new(); 

    // Extract basic block names
    for entry in &entries {
        bb_names.push(entry.1.clone());
    }

    // If one of the labels is equal to the previous one, then update the instruction
    let previous_bb_name_as_string = String::from_str(previous_bb_name)
    .expect("Failed to convert previous_bb_name to String.");
    if bb_names.contains(&previous_bb_name_as_string) {

        // Create builder for new Phi instruction
        let builder = context.create_builder();
        builder.position_at(phi_bb, instr);

        // Extract phi type
        let phi_type = match instr.get_type() {
            ArrayType(t) => BasicTypeEnum::ArrayType(t),
            FloatType(t) => BasicTypeEnum::FloatType(t),
            IntType(t) => BasicTypeEnum::IntType(t),
            PointerType(t) => BasicTypeEnum::PointerType(t),
            StructType(t) => BasicTypeEnum::StructType(t),
            VectorType(t) => BasicTypeEnum::VectorType(t),
            other_type => panic!("Expected BasicType, found {}", other_type),
        };

        let phi_name = format!("phi{}", phi_counter);
        *phi_counter += 1;
        let new_phi: PhiValue = builder.build_phi(phi_type, &phi_name)
        .expect("Failed to build phi value.");

        // Iterate over the entries of the old phi instructions to build the new one
        for entry in entries {

            // Unpack entry
            let entry_value: BasicValueEnum<'_> = entry.0;
            let entry_bb_name: String= entry.1.clone();

            // If the entry reference is equal to the previous reference then change it with the new block
            if entry_bb_name == *previous_bb_name {

                new_phi.add_incoming(&[(&entry_value, *continue_block)]);

            } else {    // else put the entry back into the new phi

                // Find the basic block with the entry_bb_reference
                for bb in function.get_basic_block_iter() {

                    let bb_name = bb.get_name().to_str().unwrap(); 

                    if bb_name == entry_bb_name {

                            new_phi.add_incoming(&[(&entry_value, bb)]);

                    }

                }

            }

        }

        instr.replace_all_uses_with(&new_phi.as_instruction());
        instr.erase_from_basic_block();
            
    }

}

/// Finds updates all phi instructions in the basic block bb
fn _update_phi_in_branch(
    context: &Context, 
    function: &FunctionValue, 
    bb: &BasicBlock, 
    previous_bb_name: &str, 
    new_bb: &BasicBlock, 
    phi_counter: &mut u32) {

    for instr in bb.get_instructions() {

        if instr.get_opcode() == Phi {

            _update_phi(context, function,*bb, &instr, previous_bb_name, new_bb, phi_counter);

        }

    }

}

/// When a block is split in two blocks (block and continue_block), we have to
/// update the phi instructions that might be present in other blocks of the LLVM
/// IR module.  To find phi instructions to update, we look for branch
/// instructions in the newly created block, and explore the target blocks. If
/// there is a phi instruction in this target blocks, we should update the
/// predecessors.
// TODO: Build tests for this function
fn _check_phi(
    context: &Context, 
    function: &FunctionValue, 
    continue_block: &BasicBlock, 
    previous_bb_name: &str, 
    phi_counter: &mut u32) {

    // Look for branch instructions
    for instr in continue_block.get_instructions() {

        if instr.get_opcode() == Br {

            // Unconditional branch
            if instr.get_num_operands() == 1 {

                let bb: BasicBlock<'_> = instr.get_operand(0)
                .expect("Failed to get branch operand at index 0.")
                .right()
                .expect("Expected BasicBlock, found BasicValueEnum.");

                _update_phi_in_branch(context, function, &bb, previous_bb_name, continue_block, phi_counter);
                
            } else {    // Conditional branch

                // Look for phi instructions in the first target blocks
                let bb_1: BasicBlock<'_> = instr.get_operand(1)
                .expect("Failed to get operand at index 1.")
                .right()
                .expect("Expected BasicBlock, found BasicValueEnum.");

                _update_phi_in_branch(context, function, &bb_1, previous_bb_name, continue_block, phi_counter);

                // Look for phi instructions in the second target blocks
                let bb_2: BasicBlock<'_> = instr.get_operand(2)
                .expect("Failed to get operand at index 2.")
                .right()
                .expect("Expected BasicBlock, found BasicValueEnum.");

                _update_phi_in_branch(context, function, &bb_2, previous_bb_name, continue_block, phi_counter);

            }

        }

    }

}

fn _handle_store_or_load(
    context: &Context, 
    function: &FunctionValue,
    instr: &InstructionValue, 
    protected_mem: (GlobalValue, GlobalValue),
    abort_bb: &BasicBlock,
    prev_bb: &BasicBlock, 
    new_bb_name: &str,
    current_block_name: &mut String,
    phi_counter: &mut u32) -> Result<(), String>{

    let new_bb: BasicBlock<'_> = context.insert_basic_block_after(*prev_bb, new_bb_name);

    // Create a new builder and position it before the instruction
    let builder: Builder<'_> = context.create_builder();
    builder.position_before(instr);

    let operand_index = match instr.get_opcode() {
        Load => 0,
        Store => 1,
        other => return Err(format!("Expected Load or Store, found {:?}", other))
    };

    // Extract pointer values
    let accessed_ptr_val: PointerValue<'_> = match instr.get_operand(operand_index).unwrap().unwrap_left() {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("Failed to extract accessed pointer value.".to_string())
    };

    // Extract alignment 
    let alignment_as_int_value: IntValue<'_> = context.i64_type().const_int(
        instr.get_alignment().expect("Failed to get alignment") as u64, false
        );

    _build_check(
        context, 
        builder, 
        protected_mem,
        (accessed_ptr_val, alignment_as_int_value),
        abort_bb,
        new_bb,
        new_bb_name)?;

    // Move instructions to the continue_block
    _move_instructions(context, instr, &new_bb);

    // Check if there is a branch in the new block.
    // If there is, check if there are phi instructions
    // in the target blocks. If there are, update previous blocks.
    _check_phi(context, function, &new_bb, current_block_name, phi_counter);
    *current_block_name = new_bb_name.to_string();

    Ok(())
}

/// Given a LLVM function adds runtime memory checks
pub fn instrument<'a>(
    function_name: &str, 
    context: &'a Context, 
    module: &Module<'a>,
    static_analysis: bool) -> Result<(), String> {

    // Retrieve function value
    let function = module.get_function(function_name).unwrap();

    // Set a name for every basic block in the code
    for (i, bb) in function.get_basic_blocks().into_iter().enumerate() {
        let name = format!("bb{}", i);
        bb.set_name(&name);
    }

    // Define the type of the abort function: fn() -> void
    let abort_type = context.void_type().fn_type(&[], false);

    // Create the abort function in the module
    let abort_func = module.add_function("abort", abort_type, None);

    // ***** Append abort block ***** //
    let abort_bb: BasicBlock<'_> = context.append_basic_block(function, "abort");

    // Create builder and position at the end of the abort basic block
    let abort_builder: Builder<'_> = context.create_builder();
    abort_builder.position_at_end(abort_bb);

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
    let protected_ptr: GlobalValue<'_> = module.add_global(pointer_type, None, "protected_ptr");
    let protected_offset: GlobalValue<'_> = module.add_global(i64_type, None, "protected_offset");
    let protected_mem: (GlobalValue<'_>, GlobalValue<'_>) = (protected_ptr, protected_offset);

    // Initialize globals
    let null_ptr: PointerValue<'_> = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
    protected_ptr.set_initializer(&null_ptr);

    let zero_offset = i64_type.const_int(0, false);
    protected_offset.set_initializer(&zero_offset);

    // Stack values
    let stack_values: Vec<PointerValue> = Vec::new();

    // * Internal state for static analysis * //
    let mut protected_mem_static: (Option<PointerValue>, Option<u64>) = (None, None);

    // Count the number of load and store instructions, to give names to blocks later
    let mut load_counter: u32 = 0;
    let mut store_counter: u32 = 0;

    // Count the number of stack allocations instructions
    let mut alloca_counter: u32 = 0;

    // Count the number of updated phi instructions, to give names later
    let mut phi_counter: u32 = 0;

    // Iterate over the basic blocks in the function
    for basic_block in function.get_basic_blocks() {

        let mut current_block_name: String = basic_block.get_name().to_str()
        .expect("Failed to get basic block name.").to_string();

        // Iterate over the instructions in the basic block
        let instructions = basic_block.get_instructions();
        for instr in instructions {

            match instr.get_opcode() {

                Call => {
                    // Check if it is the call to `utx1`
                    if instr.to_string().contains("utx0") {         
                        // remove utx0 call
                        instr.erase_from_basic_block();
                    } else if instr.to_string().contains("utx1") {     // Check if it is the call to `utx1`, Not sure if this is safe

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

                        if static_analysis {
                            let (ptr, offset) = static_checks::handle_utx1(instr);
                            protected_mem_static = (Some(ptr), Some(offset));
                        }

                        // remove utx1 call
                        instr.erase_from_basic_block();

                    }

                }

                Load => {

                    // If it's stack skip
                    if instr.print_to_string().to_string().contains("stack") {
                        continue;
                    }

                    // Create the block to store the rest of the code
                    let new_bb_name = format!("load{}", load_counter);
                    load_counter += 1;
                    
                    if static_analysis {

                        let alignment: u32 = instr.get_alignment()
                        .expect(&format!("Failed to get the alignment of instruction {:?}", instr));

                        let ptr: PointerValue = match instr.get_operand(0).unwrap().unwrap_left() { 
                            PV(ptr) => ptr,
                            other => panic!("Expected PointerValue, found {:?}", other)
                        };

                        if static_checks::is_address_protected(module.clone(), &protected_mem_static, ptr, alignment as u64) {
                            continue;
                        }

                    }

                    _handle_store_or_load(
                        context, 
                        &function, 
                        &instr, 
                        protected_mem, 
                        &abort_bb, 
                        &basic_block, 
                        &new_bb_name, 
                        &mut current_block_name, 
                        &mut phi_counter)?;

                }

                Store => {

                    // If it's stack skip
                    if instr.print_to_string().to_string().contains("stack") {
                        continue;
                    }

                    // Create the block to store the rest of the code
                    let new_bb_name: String = format!("store{}", store_counter);
                    store_counter += 1;

                    if static_analysis {

                        let alignment: u32 = instr.get_alignment()
                        .expect(&format!("Failed to get the alignment of instruction {:?}", instr));

                        let ptr: PointerValue = match instr.get_operand(1).unwrap().unwrap_left() { 
                            PV(ptr) => ptr,
                            other => panic!("Expected PointerValue, found {:?}", other)
                        };

                        if static_checks::is_address_protected(module.clone(), &protected_mem_static, ptr, alignment as u64) {
                            continue;
                        }

                    }

                    _handle_store_or_load(
                        context, 
                        &function, 
                        &instr, 
                        protected_mem, 
                        &abort_bb, 
                        &basic_block, 
                        &new_bb_name, 
                        &mut current_block_name, 
                        &mut phi_counter)?;

                }

                Alloca => {
                    instr.set_name(&format!("stack_{}", alloca_counter));
                    alloca_counter += 1;
                }

                _ => {}

            }

        }

    }

    Ok(())
}
