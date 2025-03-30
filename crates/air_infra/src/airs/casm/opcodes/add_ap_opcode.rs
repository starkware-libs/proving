use std::collections::BTreeSet;

use compiled_casm_air::compiled_structs::TraceType;
use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm
/// - ap += [fp/ap + offset]
#[derive(Clone, Debug, InstDef)]
pub struct AddApOpcode {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AddApOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: None,
            op1_base_fp: None,
            op1_base_ap: None,
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

impl AirFn for AddApOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Decode the instruction.
        let flag_sets_of_sum_1 = BTreeSet::from([BTreeSet::from([
            FLAG_OP1_IMM_INDEX,
            FLAG_OP1_BASE_FP_INDEX,
            FLAG_OP1_BASE_AP_INDEX,
        ])]);
        let ([_, _, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), Some(-1), None],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1,
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        let flag_op1_imm = flags[FLAG_OP1_IMM_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();

        ab.constrain(
            flag_op1_imm.clone() * (const_expr!(1) - offset2.clone()),
            "if imm then offset2 is 1",
        );

        let mem1_base = ab.assign(
            &mut (flag_op1_imm.clone() * casm_state.pc().var
                + flag_op1_base_fp * casm_state.fp().var
                + flag_op1_base_ap * casm_state.ap().var),
            "mem1_base",
        );

        let op1 = self
            .memory
            .read_rel_imm(ab, CasmAddress::new(mem1_base + offset2, "op1"));

        CasmStateVar::new(
            casm_state.pc().var + (const_expr!(1) + flag_op1_imm),
            casm_state.ap().var + op1,
            casm_state.fp().var,
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
