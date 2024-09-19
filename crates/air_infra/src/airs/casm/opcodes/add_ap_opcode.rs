use super::super::casm_state::*;
use super::super::common::*;
use super::decode_instruction::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm
/// - ap += [fp/ap + offset]
#[derive(Clone, Debug)]
pub struct AddAp {
    pub is_imm: bool,
    pub op1_base_fp: bool,
    pub memory: Felt252IdMemory,
}

impl AddAp {
    pub fn get_flags(&self) -> Flags {
        assert!(
            !self.is_imm || !self.op1_base_fp,
            "FLAG_OP1_IMM and FLAG_OP1_BASE_FP cannot be set at the same time."
        );
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: if self.is_imm { Some(true) } else { Some(false) },
            op1_base_fp: Some(self.op1_base_fp),
            op1_base_ap: if self.is_imm {
                Some(false)
            } else {
                Some(!self.op1_base_fp)
            },
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
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        // Decode the instruction.
        let offset2 = if self.is_imm { Some(1) } else { None };
        let ([_, _, offset2], _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), Some(-1), offset2],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc.clone(),
        );

        let op1 = if self.is_imm {
            self.memory
                .read_rel_imm(ab, casm_state.pc.clone() + const_expr!(1))
        } else {
            let mem1_base = if self.op1_base_fp {
                casm_state.fp.clone()
            } else {
                casm_state.ap.clone()
            };
            self.memory.read_rel_imm(ab, mem1_base + offset2)
        };

        CasmStateVar::new(
            casm_state.pc + const_expr!(2),
            casm_state.ap + op1,
            casm_state.fp,
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
