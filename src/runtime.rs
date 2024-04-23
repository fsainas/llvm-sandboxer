//! Adds runtime safeguards to llvm micro-transactions.

// External crates
use either::*;

// Inkwell imports
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, GlobalValue, InstructionValue, FunctionValue};
use inkwell::values::{IntValue, PointerValue, PhiValue};
use inkwell::IntPredicate::*;
use inkwell::values::AnyValueEnum;
use inkwell::values::AnyValue;

use crate::runtime::BasicValueEnum::IntValue as IV;

extern crate llvm_sys as llvm;

use regex::Regex;


// Instruction opcodes
use inkwell::values::InstructionOpcode::{Call, Load, Store, Phi, Br};

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
    protected_ptr: GlobalValue,
    protected_offset: GlobalValue,
    accessed_ptr_val: PointerValue,
    alignment_as_int_value: IntValue,
    abort_block: BasicBlock,
    continue_block: BasicBlock,
    block_name: &str
    ) -> Result<(), String> {
    let i64_type = context.i64_type();
    let ptr_type = context.i8_type().ptr_type(inkwell::AddressSpace::default());

    // Load pointer value from @protected_ptr
    let protected_ptr_val: PointerValue = match builder.build_load(
        ptr_type,
        protected_ptr.as_pointer_value(),
        &format!("protected_ptr_{}", block_name),
        ) {
        Ok(value) => value.into_pointer_value(),
        Err(_) => return Err("Failed to load value for 'protected_offset_val'".to_string()),
    };

    // Check if @protected_ptr is null
    let null_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
    let protected_is_null = builder.build_int_compare(
        EQ, 
        protected_ptr_val, 
        null_ptr,
        &format!("protected_is_null_{}", block_name)
        ).map_err(|e| format!("Failed to build integer comparison for 'accessed_ptr_val' < 'protected_ptr_val': {:?}", e))?;

    // Compare accessed pointer value with protected pointer value
    let accessed_lt_protected = builder.build_int_compare(
        SLT, 
        accessed_ptr_val, 
        protected_ptr_val, 
        &format!("accessed_lt_protected_{}", block_name)
        ).map_err(|e| format!("Failed to build integer comparison for 'accessed_ptr_val' < 'protected_ptr_val': {:?}", e))?;

    // Load protected offset value
    let protected_offset_val = match builder.build_load(
        i64_type,
        protected_offset.as_pointer_value(),
        &format!("protected_offset_{}", block_name),
        ) {
        Ok(value) => value.into_int_value(),
        Err(_) => return Err("Failed to load value for 'protected_offset_val'".to_string()),
    };

    unsafe {

        // Perform pointer arithmetic for last protected pointer value
        let last_protected_ptr_val = builder.build_gep(
            ptr_type, 
            protected_ptr_val, 
            &[protected_offset_val], 
            &format!("last_protected_ptr_{}", block_name)
            ).map_err(|e| format!("Failed to build pointer arithmetic for 'last_protected_ptr_val': {:?}", e))?;

        // Perform pointer arithmetic for last accessed pointer value
        let last_accessed_ptr_val = builder.build_gep(
            ptr_type, 
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
        let check_range = builder.build_or(
            accessed_lt_protected, 
            last_acc_gt_last_prot, 
            &format!("check_range_{}", block_name)
            ).map_err(|e| format!("Failed to build logical OR operation for 'accessed_lt_protected' || 'last_acc_gt_last_prot': {:?}", e))?;

        // Build logical OR operation for checks
        let check = builder.build_or(
            protected_is_null, 
            check_range, 
            &format!("check_{}", block_name)
            ).map_err(|e| format!("Failed to build logical OR operation for 'accessed_lt_protected' || 'last_acc_gt_last_prot': {:?}", e))?;

        // Create the instruction that evaluates comparison and chooses to abort or continue
        builder.build_conditional_branch(check, abort_block, continue_block)
            .map_err(|e| format!("Failed to build conditional branch: {:?}", e))?;

    }

    Ok(())
}



// TODO: write documentation
fn _get_phi_entries(instr: &InstructionValue) -> Vec<(String, String)> {

    if instr.get_opcode() != Phi {
        panic!("Instruction is not Phi!");
    }

    // TODO: Add reason
    let instr_as_llvmstring = instr.print_to_string();
    let instr_as_str = instr_as_llvmstring.to_str().expect("REASON");

    // To store the Phi entries
    let mut entries: Vec<(String, String)> = Vec::new();

    // Define a regular expression pattern to match labels
    // Pattern for a phi entry with reference value e.g. [ %1, %0 ]
    let ref_value_pattern: Regex = Regex::new(r" %(\w)+, %(\w)+").unwrap();

    for capture in ref_value_pattern.captures_iter(instr_as_str) {

        let binding = capture[0][1..].to_string();
        let mut parts = binding.split(',');

        // TODO: add msg
        let value = parts.next().expect("msg");
        let bb_name = parts.next().expect("msg");

        entries.push((
            value.to_string().replace(" ", ""), 
            bb_name.to_string().replace(" %", "")));

    }

    // Pattern for a phi entry with immediate value e.g. [ 1, %0 ]
    let imm_value_pattern: Regex = Regex::new(r" (\w)+, %(\w)+").unwrap();

    for capture in imm_value_pattern.captures_iter(instr_as_str) {

        let binding = capture[0][1..].to_string();
        let mut parts = binding.split(',');

        // TODO: add msg
        let value = parts.next().expect("msg");
        let bb_name = parts.next().expect("msg");

        entries.push((
            value.to_string().replace(" ", ""), 
            bb_name.to_string().replace(" %", "")));

    }

    entries

}

// When a block is split in two blocks (block and continue_block), we have to
// update the phi instructions that might be present in other blocks of the LLVM
// IR module.  To find phi instructions to update, we look for branch
// instructions in the newly created block, and explore the target blocks. If
// there is a phi instruction in this target blocks, we should update the
// predecessors.
// TODO: Build tests for this function
fn _check_phi(context: &Context, continue_block: &BasicBlock, function: &FunctionValue, previous_label: String) {

    // Look for branch instructions
    for instr in continue_block.get_instructions() {

        match instr.get_opcode() {

            Br => {
                // Look for phi instructions in the first target blocks
                let Some(Right(basic_block_1)) = instr.get_operand(1) else {todo!();};

                // Look for Phi instructions
                for instr_1 in basic_block_1.get_instructions() {

                    match instr_1.get_opcode() {

                        Phi => {

                            // ! No way to easily convert instructions with PHI opcode to phi_values
                            //let phi_value = instr_1.as_any_value_enum().into_phi_value();

                            // Get labels of previous blocks of the Phi instruction
                            // TODO: expect msg
                            // This seems the only way to obtain predecessors blocks
                            let entries: Vec<(String, String)> = _get_phi_entries(&instr_1);

                            let mut bb_names: Vec<String> = Vec::new(); 

                            // Extract basic block names
                            for entry in &entries {
                                bb_names.push(entry.1.clone());
                            }

                            // If one of the labels is equal to the previous one, then update the instruction
                            if bb_names.contains(&previous_label) {

                                // Create builder for new Phi instruction
                                let builder = context.create_builder();
                                builder.position_at(basic_block_1, &instr_1);

                                // TODO: change the type to the same as the previous phi
                                // TODO: expect msg
                                let new_phi: PhiValue = builder.build_phi(context.i64_type(), "phi").expect("REASON");

                                println!("{:?}", continue_block);

                                for entry in entries {
                                    let entry_bb_value = entry.0.clone();
                                    let entry_bb_name = entry.1.clone();
                                    if entry.1 == previous_label.to_string() {
                                        if let IV(value) = instr_1.get_operand(0).expect("msg").left().expect("msg") {
                                            new_phi.add_incoming(&[
                                                //(entry_bb_value, bb)
                                                (&value, *continue_block)
                                            ]);
                                        }
                                    } else {
                                        for bb in function.get_basic_block_iter() {
                                            let bb_name = bb.get_name().to_str().unwrap(); 

                                            if bb_name == entry.1 {
                                                if let IV(value) = instr_1.get_operand(0).expect("msg").left().expect("msg") {
                                                    new_phi.add_incoming(&[
                                                        //(entry_bb_value, bb)
                                                        (&value, bb)
                                                    ]);
                                                }
                                            }
                                            // TODO: handle else here
                                        }
                                    }
                                    instr_1.replace_all_uses_with(&new_phi.as_instruction());
                                }

                                instr_1.remove_from_basic_block();
                                
                            }

                        }

                        _ => ()

                    }

                }

                let Some(Right(basic_block_2)) = instr.get_operand(2) else {todo!();};

                // TODO: add second branch
                // Look for Phi instructions
                for instr_1 in basic_block_2.get_instructions() {

                    match instr_1.get_opcode() {

                        Phi => {

                            // ! No way to easily convert instructions with PHI opcode to phi_values
                            //let phi_value = instr_1.as_any_value_enum().into_phi_value();

                            // Get labels of previous blocks of the Phi instruction
                            // TODO: expect msg
                            // This seems the only way to obtain predecessors blocks
                            let entries: Vec<(String, String)> = _get_phi_entries(&instr_1);

                            let mut bb_names: Vec<String> = Vec::new(); 

                            // Extract basic block names
                            for entry in &entries {
                                bb_names.push(entry.1.clone());
                            }

                            // If one of the labels is equal to the previous one, then update the instruction
                            if bb_names.contains(&previous_label) {

                                // Create builder for new Phi instruction
                                let builder = context.create_builder();
                                builder.position_at(basic_block_1, &instr_1);

                                // TODO: change the type to the same as the previous phi
                                // TODO: expect msg
                                //instr_1.set_name("phi");
                                //instr_1.remove_from_basic_block();
                                let new_phi: PhiValue = builder.build_phi(context.i64_type(), "phi").expect("REASON");

                                for entry in entries {
                                    let entry_bb_value = entry.0.clone();
                                    let entry_bb_name = entry.1.clone();
                                    if entry.1 == previous_label.to_string() {
                                        if let IV(value) = instr_1.get_operand(0).expect("msg").left().expect("msg") {
                                            new_phi.add_incoming(&[
                                                //(entry_bb_value, bb)
                                                (&value, *continue_block)
                                            ]);
                                        }
                                        //new_phi.add_incoming(&[
                                        //    (instr_1.get_operand(1), *continue_block)
                                        //]);
                                    } else {
                                        for bb in function.get_basic_block_iter() {
                                            let bb_name = bb.get_name().to_str().unwrap(); 

                                            if bb_name == entry.1 {
                                                if let IV(value) = instr_1.get_operand(0).expect("msg").left().expect("msg") {
                                                    new_phi.add_incoming(&[
                                                        //(entry_bb_value, bb)
                                                        (&value, bb)
                                                    ]);
                                                }
                                            }
                                            // TODO: handle else here
                                        }
                                    }

                                    instr_1.replace_all_uses_with(&new_phi.as_instruction());
                                }

                                instr_1.remove_from_basic_block();
                            }

                        }

                        _ => ()

                    }

                }
            }

            _ => ()

        }

    }

}

/// Given a LLVM function adds runtime memory checks
pub fn instrument<'a>(
    function_name: &str, 
    context: &'a Context, 
    module: &Module<'a>) -> Result<(), String> {

    // Retrieve function value
    let function = module.get_function(function_name).unwrap();

    // Set a name for every basic block in the code
    let mut block_counter = 0;
    for bb in function.get_basic_blocks() {
        let name = format!("bb{}", block_counter);
        bb.set_name(&name);
        block_counter += 1;
    }

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

        let mut current_block_name: String = basic_block.get_name().to_str().unwrap().to_string();

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
                        protected_ptr, 
                        protected_offset,
                        accessed_ptr_val, 
                        alignment_as_int_value,
                        abort_block,
                        continue_block,
                        &block_name)?;

                    // Move instructions to the continue_block
                    _move_instructions(context, instr, &continue_block);

                    // Check if there is a branch in the new block.
                    // If there is, check if there are phi instructions
                    // in the target blocks. If there are, update previous blocks.
                    //let block_name = format!("{}", load_counter);
                    //basic_block.set_name(block_name)
                    _check_phi(context, &continue_block, &function, current_block_name.clone());
                    current_block_name = continue_block.get_name().to_str().unwrap().to_string();
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
                        protected_ptr, 
                        protected_offset,
                        accessed_ptr_val, 
                        alignment_as_int_value,
                        abort_block,
                        continue_block,
                        &block_name)?;

                    // Move instructions to the continue_block
                    _move_instructions(context, instr, &continue_block);

                    // Check if there is a branch in the new block.
                    // If there is, check if there are phi instructions
                    // in the target blocks. If there are, update previous blocks.
                    // ? How do I get the basic block label/name/identifier? 
                    _check_phi(context, &continue_block, &function, current_block_name.clone());
                    current_block_name = continue_block.get_name().to_str().unwrap().to_string();

                }

                _ => {}

            }

        }

    }

    Ok(())
}
