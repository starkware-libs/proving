use std::collections::BTreeSet;

use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::read_u32::*;
use crate::casm::common::*;
use crate::casm::decode_instruction::decode_inst::*;

// [h_pointer, message_pointer, new_state_pointer]
pub type BlakePointers = [CasmAddress; 3];

// [ap_update_add_1, is_last_block]
pub type BlakeFlags = [BoolExpr; 2];

#[derive(Debug, Serialize, Default)]
pub struct DecodeBlakeOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl DecodeBlakeOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: None,
            op1_imm: Some(false),
            op1_base_fp: None,
            op1_base_ap: None,
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: None,
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

/// Decodes the Blake opcode as follows:
/// - `op0` = `h_pointer`
/// - `op1` = `message_pointer`
/// - `dst` = `t`
/// - `[ap]` = `new_state_pointer`
///
/// Adds the relevant constraints to ensure a defined behavior.
impl AirFn for DecodeBlakeOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = (BlakePointers, UInt32Expr, BlakeFlags);

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("pc".to_string()), Some("ap".to_string()), Some("fp".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Decode the instruction.
        let flag_sets_of_sum_1 =
            BTreeSet::from([BTreeSet::from([FLAG_OP1_BASE_FP_INDEX, FLAG_OP1_BASE_AP_INDEX])]);
        let ([offset0, offset1, offset2], flags, opcode_extension) = air_builder.call(
            &DecodeInstruction {
                const_offsets: [None, None, None],
                const_flags: self.get_flags(),
                const_opcode_extension: None,
                flag_sets_of_sum_1,
                memory: self.memory.clone(),
            },
            casm_state.pc(),
        );

        // Read the non-constant flags.
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        air_builder.constrain(
            (opcode_extension.clone() - OpcodeExtension::Blake.into())
                * (opcode_extension.clone() - OpcodeExtension::BlakeFinalize.into()),
            "OpcodeExtension is either Blake or BlakeFinalize",
        );

        // Read h pointer.
        let mem0_base = air_builder.assign(
            &mut (flag_op0_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_op0_base_fp) * casm_state.ap().var),
            "mem0_base",
        );
        let h_pointer =
            self.memory.read_address(air_builder, CasmAddress::new(mem0_base + offset1, "op0"));

        // Read message pointer.
        let mem1_base = air_builder.assign(
            &mut (flag_op1_base_fp * casm_state.fp().var + flag_op1_base_ap * casm_state.ap().var),
            "mem1_base",
        );
        let message_pointer =
            self.memory.read_address(air_builder, CasmAddress::new(mem1_base + offset2, "op1"));

        // Read new state pointer.
        let new_state_pointer = self.memory.read_address(air_builder, casm_state.ap());

        // Read t.
        let read_u32 = &ReadU32 { memory: self.memory.clone() };
        let mem_dst_base = air_builder.assign(
            &mut (flag_dst_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap().var),
            "mem_dst_base",
        );
        let t = air_builder.call(read_u32, CasmAddress::new(mem_dst_base + offset0, "dst"));

        (
            [h_pointer, message_pointer, new_state_pointer],
            t,
            [
                BoolExpr::from(flag_ap_update_add_1),
                BoolExpr::from(opcode_extension - OpcodeExtension::Blake.into()),
            ],
        )
    }
}
