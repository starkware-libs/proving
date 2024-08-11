use crate::airs::casm::read_small_felt252::ReadSmallFelt252;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::FELT252_BITS_PER_WORD;

use super::check_instruction::*;
use super::common::*;

// Macros
use crate::const_expr;

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm

#[derive(Clone, Debug)]
pub struct AddAp {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AddAp {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: Some(true),
            op1_base_fp: Some(false),
            op1_base_ap: Some(false),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(true),
            ap_update_add_1: Some(false),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for AddAp {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Check the instruction.
        ab.call(
            &CheckInstruction {
                const_offsets: [Some(-1), Some(-1), Some(1)],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Fetch the immediate value.
        let imm = ab
            .call(
                &ReadSmallFelt252 {
                    // TODO: Read immediate <= Memory size.
                    num_bits: FELT252_BITS_PER_WORD,
                    memory: self.memory.clone(),
                },
                pc.clone() + const_expr!(1),
            )
            .get_felt(0);

        [pc + const_expr!(2), ap + imm, fp]
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for AddAp {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
