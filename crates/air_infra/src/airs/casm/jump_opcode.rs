use std::collections::BTreeMap;

use crate::airs::casm::read_small_felt252::ReadSmallFelt252;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::FELT252_BITS_PER_WORD;
use crate::core::variables::AirVar;

use super::check_instruction::*;
use super::common::*;
use super::read_addr::ReadAddr;

// Macros
use crate::const_expr;

/// The jump opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm
/// - jump abs [ap + offset]
/// - jump abs [fp + offset]

#[derive(Clone, Debug)]
pub struct JumpOpcode {
    pub is_rel: bool,
    pub flag_op1_base_fp: bool,
    pub flag_ap_update_add_1: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl JumpOpcode {
    pub fn get_flags(&self) -> Flags {
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.flag_op1_base_fp);
            false
        } else {
            !self.flag_op1_base_fp
        };
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: Some(self.is_rel),
            op1_base_fp: Some(self.flag_op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(self.flag_ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JumpOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset0 = offset_as_u16(-1);
        let offset1 = offset_as_u16(-1);
        let offset2 = if self.is_rel {
            Some(offset_as_u16(1))
        } else {
            None
        };

        // Create the flags.
        let flags = self.get_flags();

        // Check the instruction.
        let ([_, _, offset2], _) = ab.call(
            &CheckInstruction {
                const_offsets: [Some(offset0), Some(offset1), offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Calculate the next pc
        let next_pc = if self.is_rel {
            pc.clone()
                + ab.call(
                    &ReadSmallFelt252 {
                        num_bits: FELT252_BITS_PER_WORD,
                        memory: self.memory.clone(),
                    },
                    pc + const_expr!(1),
                )
                .as_felts()[0]
                    .clone()
        } else {
            let mem1_base = if self.flag_op1_base_fp {
                fp.clone()
            } else {
                ap.clone()
            };
            ab.call(
                &ReadAddr {
                    memory: self.memory.clone(),
                },
                mem1_base + offset2,
            )
        };

        // Calculate the next ap
        let next_ap = if self.flag_ap_update_add_1 {
            ap + const_expr!(1)
        } else {
            ap
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            ("is_rel".to_string(), self.is_rel.to_string()),
            (
                "flag_op1_base_fp".to_string(),
                self.flag_op1_base_fp.to_string(),
            ),
            (
                "flag_ap_update_add".to_string(),
                self.flag_ap_update_add_1.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for JumpOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
