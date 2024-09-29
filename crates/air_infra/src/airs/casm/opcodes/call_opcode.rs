use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::super::decode_instruction::decode_inst::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::airs::memory::felt252_id_memory_verify::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

/// The call opcode.
/// Implements the Cairo0 instructions:
/// - call rel imm
/// - call abs [ap + offset]
/// - call abs [fp + offset]

#[derive(Clone, Debug, InstDef)]
pub struct CallOpcode {
    pub is_rel: bool,
    pub op1_base_fp: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl CallOpcode {
    pub fn get_flags(&self) -> Flags {
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.op1_base_fp);
            false
        } else {
            !self.op1_base_fp
        };
        Flags {
            dst_base_fp: Some(false),
            op0_base_fp: Some(false),
            op1_imm: Some(self.is_rel),
            op1_base_fp: Some(self.op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
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
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset2 = if self.is_rel { Some(1) } else { None };

        // Check the instruction.
        let ([_, _, offset2], _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(0), Some(1), offset2],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc.clone(),
        );

        // Push fp.
        ab.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                casm_state.ap.clone(),
                Felt252Expr::from(vec![casm_state.fp.clone()]),
            ),
        );

        // Push pc + instruction_size.
        ab.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                casm_state.ap.clone() + const_expr!(1),
                Felt252Expr::from(vec![
                    (casm_state.pc.clone() + const_expr!(1 + (self.is_rel as u32))),
                ]),
            ),
        );

        // Update pc.
        let next_pc = if self.is_rel {
            casm_state.pc.clone() + self.memory.read_rel_imm(ab, casm_state.pc + const_expr!(1))
        } else {
            let mem1_base = if self.op1_base_fp {
                casm_state.fp.clone()
            } else {
                casm_state.ap.clone()
            };
            self.memory.read_address(ab, mem1_base + offset2)
        };

        CasmStateVar::new(
            next_pc,
            casm_state.ap.clone() + const_expr!(2),
            casm_state.ap + const_expr!(2),
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
