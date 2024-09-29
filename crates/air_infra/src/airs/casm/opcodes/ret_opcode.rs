use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

pub const RET_FLAGS: Flags = Flags {
    dst_base_fp: Some(true),
    op0_base_fp: Some(true),
    op1_imm: Some(false),
    op1_base_fp: Some(true),
    op1_base_ap: Some(false),
    res_add: Some(false),
    res_mul: Some(false),
    pc_update_jump: Some(true),
    pc_update_jump_rel: Some(false),
    pc_update_jnz: Some(false),
    ap_update_add: Some(false),
    ap_update_add_1: Some(false),
    opcode_call: Some(false),
    opcode_ret: Some(true),
    opcode_assert_eq: Some(false),
};

#[derive(Debug, Default, InstDef)]
pub struct RetOpcode {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for RetOpcode {
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, air_builder: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        let decode_instruction = DecodeInstruction {
            const_offsets: [Some(-2), Some(-1), Some(-1)],
            const_flags: RET_FLAGS,
            memory: self.memory.clone(),
        };

        air_builder.call(&decode_instruction, casm_state.pc);

        // Read the saved pc and fp
        let next_pc = self
            .memory
            .read_address(air_builder, casm_state.fp.clone() - const_expr!(1));

        let next_fp = self
            .memory
            .read_address(air_builder, casm_state.fp - const_expr!(2));

        CasmStateVar::new(next_pc, casm_state.ap, next_fp)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
