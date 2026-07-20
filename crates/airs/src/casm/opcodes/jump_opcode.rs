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

/// The jump opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm
/// - jump rel [ap/fp + offset2]
/// - jump abs [ap/fp + offset2]
/// - jump abs [[ap/fp + offset1] + offset2]

#[derive(Clone, Debug, Serialize)]
pub struct JumpOpcode {
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub rel: bool,
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub imm: bool,
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub double_deref: bool,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl JumpOpcode {
    pub fn get_flags(&self) -> Flags {
        assert!(
            !self.imm || !self.double_deref,
            "Cannot set flags to support double deref and immediate at the same time.",
        );
        assert!(self.rel || !self.imm, "Immediate jump must be relative.",);
        assert!(!self.double_deref || !self.rel, "Double deref jump must be absolute.",);
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: (!self.double_deref).then_some(true),
            op1_imm: Some(self.imm),
            op1_base_fp: (self.imm || self.double_deref).then_some(false),
            op1_base_ap: (self.imm || self.double_deref).then_some(false),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.rel),
            pc_update_jump_rel: Some(self.rel),
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
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset1 = if self.double_deref { None } else { Some(-1) };
        let offset2 = if self.imm { Some(1) } else { None };

        let flag_sets_of_sum_1 = if !self.imm && !self.double_deref {
            BTreeSet::from([BTreeSet::from([FLAG_OP1_BASE_FP_INDEX, FLAG_OP1_BASE_AP_INDEX])])
        } else {
            BTreeSet::new()
        };

        // Check the instruction.
        let ([_, offset1, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), offset1, offset2],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1,
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
        let mem1_base = if self.imm {
            casm_state.pc().var
        } else if self.double_deref {
            let mem0_base = ab.assign(
                &mut (op0_base_fp.clone() * casm_state.fp().var
                    + (const_expr!(1) - op0_base_fp) * casm_state.ap().var),
                "mem0_base",
            );
            self.memory.read_address(ab, CasmAddress::new(mem0_base + offset1, "mem1_base")).var
        } else {
            ab.assign(
                &mut (op1_base_fp * casm_state.fp().var + op1_base_ap * casm_state.ap().var),
                "mem1_base",
            )
        };

        let next_pc = if self.rel {
            casm_state.pc().var
                + self.memory.read_rel_imm(ab, CasmAddress::new(mem1_base + offset2, "next_pc"))
        } else {
            self.memory.read_address(ab, CasmAddress::new(mem1_base + offset2, "next_pc")).var
        };

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }

    fn name(&self) -> String {
        let name = self.get_type_name();
        let inst_def = self.get_inst_def();

        if inst_def.is_empty() {
            return format!("{name}_abs");
        }

        format!("{name}_{inst_def}")
    }
}
