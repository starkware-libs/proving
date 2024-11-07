use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;
use crate::airs::felt252_id_memory::memory::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm
/// - ap += [fp/ap + offset]
#[derive(Clone, Debug, InstDef)]
pub struct AddApOpcode {
    pub is_imm: bool,
    pub op1_base_fp: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AddApOpcode {
    pub fn get_flags(&self) -> Flags {
        assert!(
            !self.is_imm || !self.op1_base_fp,
            "FLAG_OP1_IMM and FLAG_OP1_BASE_FP cannot be set at the same time."
        );
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: if self.is_imm { Some(true) } else { Some(false) },
            op1_base_fp: Some(self.op1_base_fp),
            op1_base_ap: if self.is_imm {
                Some(false)
            } else {
                Some(!self.op1_base_fp)
            },
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
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        // Decode the instruction.
        let offset2 = if self.is_imm { Some(1) } else { None };
        let ([_, _, offset2], _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), Some(-1), offset2],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc.clone(),
        );

        let op1 = if self.is_imm {
            self.memory.read_rel_imm(
                ab,
                CasmAddress::new(casm_state.pc.value.clone() + const_expr!(1), "op1"),
            )
        } else {
            let mem1_base = if self.op1_base_fp {
                casm_state.fp.value.clone()
            } else {
                casm_state.ap.value.clone()
            };
            self.memory
                .read_rel_imm(ab, CasmAddress::new(mem1_base + offset2, "op1"))
        };

        CasmStateVar::new(
            casm_state.pc.value + (const_expr!(1) + const_expr!(self.is_imm as u32)),
            casm_state.ap.value + op1,
            casm_state.fp.value,
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
