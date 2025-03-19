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

/// The call opcode.
/// Implements the Cairo0 instructions:
/// - call rel imm
/// - call abs [ap + offset]
/// - call abs [fp + offset]

#[derive(Clone, Debug, InstDef)]
pub struct CallOpcode {
    pub rel: bool,
    pub op1_base_fp: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl CallOpcode {
    pub fn get_flags(&self) -> Flags {
        let flag_op1_base_ap = if self.rel {
            assert!(
                !self.op1_base_fp,
                "Flag op1_base_fp cannot be set for relative calls."
            );
            false
        } else {
            !self.op1_base_fp
        };
        Flags {
            dst_base_fp: Some(false),
            op0_base_fp: Some(false),
            op1_imm: Some(self.rel),
            op1_base_fp: Some(self.op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.rel),
            pc_update_jump_rel: Some(self.rel),
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
        let offset2 = if self.rel { Some(1) } else { None };

        // Check the instruction.
        let ([_, _, offset2], ..) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(0), Some(1), offset2],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1: BTreeSet::new(),
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Push fp.
        let stored_fp_address = CasmAddress::new(casm_state.ap().var, "stored_fp");
        let stored_fp = self.memory.read_address(ab, stored_fp_address);
        ab.constrain(stored_fp.var - casm_state.fp().var, "[ap] = fp");

        // Push pc + instruction_size.
        let stored_ret_pc_address =
            CasmAddress::new(casm_state.ap().var + const_expr!(1), "stored_ret_pc");
        let stored_ret_pc = self.memory.read_address(ab, stored_ret_pc_address);
        let return_pc = casm_state.pc().var + const_expr!(1 + (self.rel as u32));
        ab.constrain(stored_ret_pc.var - return_pc, "[ap+1] = return_pc");

        // Update pc.
        let next_pc = if self.rel {
            casm_state.pc().var
                + self.memory.read_rel_imm(
                    ab,
                    CasmAddress::new(casm_state.pc().var + const_expr!(1), "distance_to_next_pc"),
                )
        } else {
            let mem1_base = if self.op1_base_fp {
                casm_state.fp().var
            } else {
                casm_state.ap().var
            };
            self.memory
                .read_address(ab, CasmAddress::new(mem1_base + offset2, "next_pc"))
                .var
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
}
