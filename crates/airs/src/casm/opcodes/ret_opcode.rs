use std::collections::BTreeSet;

use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::super::decode_instruction::decode_inst::*;
use crate::casm::common::*;

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

#[derive(Debug, Default, Serialize)]
pub struct RetOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for RetOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, air_builder: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let decode_instruction = DecodeInstruction {
            const_offsets: [Some(-2), Some(-1), Some(-1)],
            const_flags: RET_FLAGS,
            const_opcode_extension: Some(OpcodeExtension::Stone),
            flag_sets_of_sum_1: BTreeSet::new(),
            memory: self.memory.clone(),
        };

        air_builder.call(&decode_instruction, casm_state.pc().clone());

        // Read the saved pc and fp
        let next_pc = self.memory.read_address(
            air_builder,
            CasmAddress::new(casm_state.fp().var - const_expr!(1), "next_pc"),
        );

        let next_fp = self.memory.read_address(
            air_builder,
            CasmAddress::new(casm_state.fp().var - const_expr!(2), "next_fp"),
        );

        CasmStateVar::new(next_pc.var, casm_state.ap().var, next_fp.var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
