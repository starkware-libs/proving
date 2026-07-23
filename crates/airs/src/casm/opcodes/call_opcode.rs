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

/// The call opcode.
/// Implements the Cairo0 instructions:
/// - call rel imm
/// - call abs [ap + offset]
/// - call abs [fp + offset]

#[derive(Clone, Debug, Serialize)]
pub struct CallOpcode {
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub rel_imm: bool,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl CallOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(false),
            op0_base_fp: Some(false),
            op1_imm: Some(self.rel_imm),
            op1_base_fp: (self.rel_imm).then_some(false),
            op1_base_ap: (self.rel_imm).then_some(false),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.rel_imm),
            pc_update_jump_rel: Some(self.rel_imm),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(false),
            opcode_call: Some(true),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for CallOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset2 = if self.rel_imm { Some(1) } else { None };

        let flag_sets_of_sum_1 = if self.rel_imm {
            BTreeSet::new()
        } else {
            BTreeSet::from([BTreeSet::from([FLAG_OP1_BASE_FP_INDEX, FLAG_OP1_BASE_AP_INDEX])])
        };

        // Check the instruction.
        let ([_, _, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(0), Some(1), offset2],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1,
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();

        // Push fp.
        let stored_fp_address = CasmAddress::new(casm_state.ap().var, "stored_fp");
        let stored_fp = self.memory.read_address(ab, stored_fp_address);
        ab.constrain(stored_fp.var - casm_state.fp().var, "[ap] = fp");

        // Push pc + instruction_size.
        let stored_ret_pc_address =
            CasmAddress::new(casm_state.ap().var + const_expr!(1), "stored_ret_pc");
        let stored_ret_pc = self.memory.read_address(ab, stored_ret_pc_address);
        let return_pc = casm_state.pc().var + const_expr!(1 + (self.rel_imm as u32));
        ab.constrain(stored_ret_pc.var - return_pc, "[ap+1] = return_pc");

        // Update pc.
        let next_pc = if self.rel_imm {
            casm_state.pc().var
                + self.memory.read_rel_imm(
                    ab,
                    CasmAddress::new(casm_state.pc().var + const_expr!(1), "distance_to_next_pc"),
                )
        } else {
            let mem1_base = ab.assign(
                &mut (flag_op1_base_fp * casm_state.fp().var
                    + flag_op1_base_ap * casm_state.ap().var),
                "mem1_base",
            );
            self.memory.read_address(ab, CasmAddress::new(mem1_base + offset2, "next_pc")).var
        };

        CasmStateVar::new(
            next_pc,
            casm_state.ap().var + const_expr!(2),
            casm_state.ap().var + const_expr!(2),
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }

    fn name(&self) -> String {
        format!("call_opcode_{}", if self.rel_imm { "rel_imm" } else { "abs" })
    }
}
