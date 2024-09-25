use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::airs::memory::felt252_id_memory_read_small::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

/// The add_small opcode.
/// Implements the Cairo0 instructions:
/// - [ap/fp + offset0] = [ap/fp + offset1] + [ap/fp + offset2]
/// - [ap/fp + offset0] = [ap/fp + offset1] + Imm
/// Where all three values are in the range [0, 2^27-1].
///
/// TODO: Update the range when the correct range is known.

#[derive(Clone, Debug, InstDef)]
pub struct AddSmallOpcode {
    pub is_imm: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AddSmallOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: None,
            op1_imm: Some(self.is_imm),
            op1_base_fp: if !self.is_imm { None } else { Some(false) },
            op1_base_ap: if !self.is_imm { None } else { Some(false) },
            res_add: Some(true),
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

impl AirFn for AddSmallOpcode {
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        let const_offsets = if self.is_imm {
            [None, None, Some(1)]
        } else {
            [None, None, None]
        };
        // Check the instruction.
        let ([offset0, offset1, offset2], flags) = ab.call(
            &DecodeInstruction {
                const_offsets,
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc.clone(),
        );

        // Read the non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        // Fetch dst - the value at the destination address for the addition
        let mem_dst_base = flag_dst_base_fp.clone() * casm_state.fp.clone()
            + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap.clone();
        let (dst_m31, _) = ab.call(
            &ReadSmall {
                memory: self.memory.clone(),
            },
            mem_dst_base + offset0,
        );

        // Fetch op0 - the first operand for the addition
        let mem0_base = flag_op0_base_fp.clone() * casm_state.fp.clone()
            + (const_expr!(1) - flag_op0_base_fp) * casm_state.ap.clone();
        let (op0_m31, _) = ab.call(
            &ReadSmall {
                memory: self.memory.clone(),
            },
            mem0_base + offset1,
        );

        // Fetch op1 - the second operand for the addition
        let mem1_base = if self.is_imm {
            casm_state.pc.clone()
        } else {
            ab.constrain(flag_op1_base_fp.clone() + flag_op1_base_ap.clone() - const_expr!(1));
            flag_op1_base_fp * casm_state.fp.clone() + flag_op1_base_ap * casm_state.ap.clone()
        };
        let (op1_m31, _) = ab.call(
            &ReadSmall {
                memory: self.memory.clone(),
            },
            mem1_base + offset2,
        );

        let res = op0_m31 + op1_m31;

        // Assert that dst == res
        ab.constrain(dst_m31 - res);

        // Calculate the next ap
        let next_ap = casm_state.ap.clone() + flag_ap_update_add_1;

        // Calculate the next pc
        let next_pc = if self.is_imm {
            casm_state.pc + const_expr!(2)
        } else {
            casm_state.pc + const_expr!(1)
        };

        CasmStateVar::new(next_pc, next_ap, casm_state.fp)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
