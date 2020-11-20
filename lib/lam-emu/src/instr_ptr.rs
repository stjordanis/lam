use super::bytecode::*;
use super::literal::*;
use super::program::*;
use std::boxed::Box;

use log::*;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct InstructionPointer {
    last_instr_ptr: Option<Box<InstructionPointer>>,
    current_module: String,
    current_label: Label,
    current_instruction: usize,
    pub instr: Instruction,
}

impl InstructionPointer {
    pub fn new(mfa: &MFA, program: &Program) -> InstructionPointer {
        trace!("Creating pointer for {:#?} in {:#?}", mfa, program);
        let module = program.modules.get(&mfa.module).unwrap();
        let function_key = (mfa.function.clone(), mfa.arity);
        let first_label = module
            .functions
            .get(&function_key)
            .unwrap_or_else(|| panic!("Could not find function : {:?}", &function_key));
        let first_instruction = module.labels[*first_label as usize].instructions[0].clone();

        InstructionPointer {
            last_instr_ptr: None,
            current_module: module.name.clone(),
            current_label: *first_label,
            current_instruction: 0,
            instr: first_instruction,
        }
    }

    pub fn get_next(&self, program: &Program) -> InstructionPointer {
        let module = program.modules.get(&self.current_module).unwrap();

        let label = (self.current_label) as usize;
        let instructions = &module.labels[label].instructions;

        let last_offset = instructions.len();

        let next_instr = self.current_instruction + 1;

        let mut next_instr_ptr = self.clone();
        if next_instr < last_offset {
            next_instr_ptr.current_instruction += 1;
            next_instr_ptr.instr =
                instructions[next_instr_ptr.current_instruction as usize].clone();
        } else if let Some(last_ptr) = &self.last_instr_ptr {
            next_instr_ptr = (**last_ptr).clone();
        } else {
            next_instr_ptr.instr = Instruction::Halt;
        }
        next_instr_ptr
    }

    pub fn next(&mut self, program: &Program) {
        *self = self.get_next(&program);
    }

    pub fn call(&mut self, program: &Program, call: &FnCall) {
        let next_ptr = self.get_next(&program);

        let module_name = call
            .module()
            .unwrap_or_else(|| next_ptr.current_module.clone());
        let module = program
            .modules
            .get(&module_name)
            .unwrap_or_else(|| panic!("Could not find module: {:?}", &module_name));
        let first_label = match call {
            FnCall::Local { label, .. } => label,
            _ => {
                let function_key = (call.function(), call.arity());
                module
                    .functions
                    .get(&function_key)
                    .unwrap_or_else(|| panic!("Could not find function : {:?}", &function_key))
            }
        };
        let first_instruction = module.labels[*first_label as usize].instructions[0].clone();

        *self = InstructionPointer {
            last_instr_ptr: Some(Box::new(next_ptr)),
            current_module: module.name.clone(),
            current_label: *first_label,
            current_instruction: 0,
            instr: first_instruction,
        }
    }

    pub fn jump_to_label(&mut self, program: &Program, label: &Label) {
        trace!("Jumping to label: {:?}", label);

        let next_ptr = self.get_next(&program);

        let module_name = self.current_module.clone();
        let module = program
            .modules
            .get(&module_name)
            .unwrap_or_else(|| panic!("Could not find module: {:?}", &module_name));
        trace!("Found module: {:?}", module_name);

        let first_instruction = module.labels[*label as usize].instructions[0].clone();
        trace!("First instruction: {:?}", first_instruction);

        *self = InstructionPointer {
            current_module: module_name,
            current_label: *label,
            current_instruction: 0,
            instr: first_instruction,
            last_instr_ptr: Some(Box::new(next_ptr)),
        }
    }

    pub fn return_to_last_instr(&mut self) {
        let last_ptr = self.last_instr_ptr.clone().unwrap();
        *self = *last_ptr;
    }
}