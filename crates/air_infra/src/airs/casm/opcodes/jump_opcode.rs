use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;

/// The jump opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm
/// - jump rel [ap/fp + offset2]
/// - jump abs [ap/fp + offset2]
/// - jump abs [[ap/fp + offset1] + offset2]

#[derive(Clone, Debug, InstDef)]
pub struct JumpOpcode {
    pub is_rel: bool,
    pub is_imm: bool,
    pub is_double_deref: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl JumpOpcode {
    pub fn get_flags(&self) -> Flags {
        assert!(
            !self.is_imm || !self.is_double_deref,
            "Cannot set flags to support double deref and immediate at the same time.",
        );
        assert!(
            self.is_rel || !self.is_imm,
            "Immediate jump must be relative.",
        );
        assert!(
            !self.is_double_deref || !self.is_rel,
            "Double deref jump must be absolute.",
        );
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: (!self.is_double_deref).then_some(true),
            op1_imm: Some(self.is_imm),
            op1_base_fp: (self.is_imm || self.is_double_deref).then_some(false),
            op1_base_ap: (self.is_imm || self.is_double_deref).then_some(false),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: None,
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JumpOpcode {
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset1 = if self.is_double_deref { None } else { Some(-1) };
        let offset2 = if self.is_imm { Some(1) } else { None };

        // Check the instruction.
        let ([_, offset1, offset2], flags) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), offset1, offset2],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Read non-constant flags
        let op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        // Calculate the next pc
        let mem1_base = if self.is_imm {
            casm_state.pc().var
        } else if self.is_double_deref {
            let mem0_base = ab.assign(
                &mut (op0_base_fp.clone() * casm_state.fp().var
                    + (const_expr!(1) - op0_base_fp) * casm_state.ap().var),
                "mem0_base",
            );
            self.memory
                .read_address(ab, CasmAddress::new(mem0_base + offset1, "mem1_base"))
                .var
        } else {
            ab.constrain(
                op1_base_fp.clone() + op1_base_ap.clone() - const_expr!(1),
                "Either flag op1_base_fp is on or flag op1_base_ap is on",
            );
            ab.assign(
                &mut (op1_base_fp * casm_state.fp().var + op1_base_ap * casm_state.ap().var),
                "mem1_base",
            )
        };

        let next_pc = if self.is_rel {
            casm_state.pc().var
                + self
                    .memory
                    .read_rel_imm(ab, CasmAddress::new(mem1_base + offset2, "next_pc"))
        } else {
            self.memory
                .read_address(ab, CasmAddress::new(mem1_base + offset2, "next_pc"))
                .var
        };

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
