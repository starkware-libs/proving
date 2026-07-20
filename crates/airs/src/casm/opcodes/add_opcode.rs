use std::collections::BTreeSet;

use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::read_small::ReadSmall;
use serde::Serialize;

use super::super::decode_instruction::decode_inst::*;
use crate::casm::common::*;
use crate::felt252_utils::verify_add252::*;

/// The add opcode.
/// Implements the Cairo0 instructions:
/// - [ap/fp + offset0] = [ap/fp + offset1] + [ap/fp + offset2]
/// - [ap/fp + offset0] = [ap/fp + offset1] + Imm
///
/// small = true : all three values are in the range [-2**29 - 1, 2**29 - 1].
/// small = false : all three values are in the range [0, 2**252 - 1].
#[derive(Clone, Debug, Serialize)]
pub struct AddOpcode {
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub small: bool,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AddOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: None,
            op1_imm: None,
            op1_base_fp: None,
            op1_base_ap: None,
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

impl AirFn for AddOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let flag_sets_of_sum_1 = BTreeSet::from([BTreeSet::from([
            FLAG_OP1_IMM_INDEX,
            FLAG_OP1_BASE_FP_INDEX,
            FLAG_OP1_BASE_AP_INDEX,
        ])]);
        // Check the instruction.
        let ([offset0, offset1, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [None, None, None],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1,
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Read the non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let flag_op1_imm = flags[FLAG_OP1_IMM_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        ab.constrain(
            flag_op1_imm.clone() * (const_expr!(1) - offset2.clone()),
            "if imm then offset2 is 1",
        );

        let mem_dst_base = ab.assign(
            &mut (flag_dst_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap().var),
            "mem_dst_base",
        );
        let mem0_base = ab.assign(
            &mut (flag_op0_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_op0_base_fp) * casm_state.ap().var),
            "mem0_base",
        );
        let mem1_base = ab.assign(
            &mut (flag_op1_imm.clone() * casm_state.pc().var
                + flag_op1_base_fp * casm_state.fp().var
                + flag_op1_base_ap * casm_state.ap().var),
            "mem1_base",
        );

        // Add Small
        if self.small {
            let (dst, _) = ab.call(
                &ReadSmall { memory: self.memory.clone() },
                CasmAddress::new(mem_dst_base + offset0, "dst"),
            );
            let (op0, _) = ab.call(
                &ReadSmall { memory: self.memory.clone() },
                CasmAddress::new(mem0_base + offset1, "op0"),
            );
            let (op1, _) = ab.call(
                &ReadSmall { memory: self.memory.clone() },
                CasmAddress::new(mem1_base + offset2, "op1"),
            );
            // Assert that dst == op0 + op1
            ab.constrain(dst - (op0 + op1), "dst equals op0 + op1");
        } else {
            // Add big
            let dst = self.memory.read_felt252(ab, CasmAddress::new(mem_dst_base + offset0, "dst"));
            let op0 = self.memory.read_felt252(ab, CasmAddress::new(mem0_base + offset1, "op0"));
            let op1 = self.memory.read_felt252(ab, CasmAddress::new(mem1_base + offset2, "op1"));
            ab.call(&VerifyAdd252 {}, [op0, op1, dst]);
        }

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        // Calculate the next pc
        let next_pc = casm_state.pc().var + const_expr!(1) + flag_op1_imm;

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
