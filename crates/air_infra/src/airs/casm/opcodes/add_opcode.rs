use std::collections::BTreeSet;

use compiled_casm_air::compiled_structs::TraceType;
use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;
use crate::airs::felt252_utils::verify_add252::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_small::*;

/// The add opcode.
/// Implements the Cairo0 instructions:
/// - [ap/fp + offset0] = [ap/fp + offset1] + [ap/fp + offset2]
/// - [ap/fp + offset0] = [ap/fp + offset1] + Imm
///
/// small = true : all three values are in the range [-2**27, 2**27 - 1].
/// small = false : all three values are in the range [0, 2**252 - 1].
#[derive(Clone, Debug, InstDef)]
pub struct AddOpcode {
    pub small: bool,
    pub imm: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AddOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: None,
            op1_imm: Some(self.imm),
            op1_base_fp: if !self.imm { None } else { Some(false) },
            op1_base_ap: if !self.imm { None } else { Some(false) },
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
        let (const_offsets, flag_sets_of_sum_1) = if self.imm {
            ([None, None, Some(1)], BTreeSet::new())
        } else {
            (
                [None, None, None],
                BTreeSet::from([BTreeSet::from([
                    FLAG_OP1_BASE_FP_INDEX,
                    FLAG_OP1_BASE_AP_INDEX,
                ])]),
            )
        };
        // Check the instruction.
        let ([offset0, offset1, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets,
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
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

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
        let mem1_base = if self.imm {
            casm_state.pc().var
        } else {
            ab.assign(
                &mut (flag_op1_base_fp * casm_state.fp().var
                    + flag_op1_base_ap * casm_state.ap().var),
                "mem1_base",
            )
        };

        // Add Small
        if self.small {
            let (dst, _) = ab.call(
                &ReadSmall {
                    memory: self.memory.clone(),
                },
                CasmAddress::new(mem_dst_base + offset0, "dst"),
            );
            let (op0, _) = ab.call(
                &ReadSmall {
                    memory: self.memory.clone(),
                },
                CasmAddress::new(mem0_base + offset1, "op0"),
            );
            let (op1, _) = ab.call(
                &ReadSmall {
                    memory: self.memory.clone(),
                },
                CasmAddress::new(mem1_base + offset2, "op1"),
            );
            // Assert that dst == op0 + op1
            ab.constrain(dst - (op0 + op1), "dst equals op0 + op1");
        } else {
            // Add big
            let dst = self
                .memory
                .read_felt252(ab, CasmAddress::new(mem_dst_base + offset0, "dst"));
            let op0 = self
                .memory
                .read_felt252(ab, CasmAddress::new(mem0_base + offset1, "op0"));
            let op1 = self
                .memory
                .read_felt252(ab, CasmAddress::new(mem1_base + offset2, "op1"));
            ab.call(&VerifyAdd252 {}, [op0, op1, dst]);
        }

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        // Calculate the next pc
        let next_pc = if self.imm {
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
