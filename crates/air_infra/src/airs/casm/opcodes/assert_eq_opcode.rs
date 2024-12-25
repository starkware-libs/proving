use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify_equal::*;

/// The assert_eq opcode.
/// Implements the Cairo0 instructions:
/// - [ap/fp + offset0] = [ap/fp + offset2]
/// - [ap/fp + offset0] = [[ap/fp + offset1] + offset2]
/// - [ap/fp + offset0] = imm

#[derive(Clone, Debug, InstDef)]
pub struct AssertEqOpcode {
    pub is_double_deref: bool,
    pub is_imm: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AssertEqOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: if self.is_double_deref {
                None
            } else {
                Some(true) // Default is fp based
            },
            op1_imm: Some(self.is_imm),
            op1_base_fp: if !self.is_double_deref && !self.is_imm {
                None
            } else {
                Some(false)
            },
            op1_base_ap: if !self.is_double_deref && !self.is_imm {
                None
            } else {
                Some(false)
            },
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: None,
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(true),
        }
    }
}

impl AirFn for AssertEqOpcode {
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        assert!(
            !(self.is_imm && self.is_double_deref),
            "Double deref and immediate can't be set together"
        );

        // Create the constant offsets.
        let offsets = if self.is_imm {
            [None, Some(-1), Some(1)]
        } else if self.is_double_deref {
            [None, None, None]
        } else {
            [None, Some(-1), None]
        };

        // Check the instruction.
        let ([offset0, offset1, offset2], flags) = ab.call(
            &DecodeInstruction {
                const_offsets: offsets,
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Read the non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        // Fetch dst
        let mem_dst_base = ab.assign(
            &mut (flag_dst_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap().var),
            "mem_dst_base",
        );

        // Find mem1_base
        let mem1_base = if self.is_double_deref {
            let mem0_base = ab.assign(
                &mut (flag_op0_base_fp.clone() * casm_state.fp().var
                    + (const_expr!(1) - flag_op0_base_fp) * casm_state.ap().var),
                "mem0_base",
            );
            self.memory
                .read_address(ab, CasmAddress::new(mem0_base + offset1, "mem1_base"))
                .var
        } else if self.is_imm {
            casm_state.pc().var
        } else {
            ab.constrain(
                flag_op1_base_fp.clone() + flag_op1_base_ap.clone() - const_expr!(1),
                "Either flag op1_base_fp is on or flag op1_base_ap is on",
            );
            ab.assign(
                &mut (flag_op1_base_fp * casm_state.fp().var
                    + flag_op1_base_ap * casm_state.ap().var),
                "mem1_base",
            )
        };

        // Assert that dst == op1
        ab.call(
            &MemVerifyEqual {
                memory: self.memory.clone(),
            },
            [
                CasmAddress::new(mem_dst_base + offset0, "dst"),
                CasmAddress::new(mem1_base + offset2, "op1"),
            ],
        );

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        // Calculate the next pc
        let next_pc = if self.is_imm {
            casm_state.pc().var + const_expr!(2)
        } else {
            casm_state.pc().var + const_expr!(1)
        };

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
