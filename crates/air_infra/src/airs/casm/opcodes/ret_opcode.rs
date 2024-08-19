use super::super::common::*;
use super::super::read_addr::*;
use super::decode_instruction::*;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

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

#[derive(Debug, Default)]
pub struct RetOpcode {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl MemoryAirFn for RetOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<Self::K, Self::V>) {
        self.memory = memory.clone();
    }
}

impl AirFn for RetOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, air_builder: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        let decode_instruction = DecodeInstruction {
            const_offsets: [Some(-2), Some(-1), Some(-1)],
            const_flags: RET_FLAGS,
            memory: self.memory.clone(),
        };

        air_builder.call(&decode_instruction, pc);

        // Read the saved pc and fp as memory addresses,
        // so we don't support values > 2**24 for them.
        let read_addr = ReadAddr {
            memory: self.memory.clone(),
        };
        let next_pc = air_builder.call(&read_addr, fp.clone() - const_expr!(1));

        let next_fp = air_builder.call(&read_addr, fp - const_expr!(2));

        [next_pc, ap, next_fp]
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
