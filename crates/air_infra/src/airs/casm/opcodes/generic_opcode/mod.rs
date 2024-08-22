pub mod decode_generic_instruction;
pub mod eval_operands;
#[allow(clippy::module_inception)]
pub mod generic_opcode;
#[cfg(test)]
pub mod generic_opcode_test;
pub mod handle_opcodes;
pub mod update_registers;
