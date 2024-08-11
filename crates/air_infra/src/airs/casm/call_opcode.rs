use indexmap::IndexMap;

use super::check_instruction::*;
use super::common::*;
use super::read_addr::*;
use super::read_small_felt252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;

// Macros
use crate::const_expr;

/// The call opcode.
/// Implements the Cairo0 instructions:
/// - call rel imm
/// - call abs [ap + offset]
/// - call abs [fp + offset]

#[derive(Clone, Debug)]
pub struct CallOpcode {
    pub is_rel: bool,
    pub flag_op1_base_fp: bool,

    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl CallOpcode {
    pub fn get_flags(&self) -> Flags {
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.flag_op1_base_fp);
            false
        } else {
            !self.flag_op1_base_fp
        };
        Flags {
            dst_base_fp: Some(false),
            op0_base_fp: Some(false),
            op1_imm: Some(self.is_rel),
            op1_base_fp: Some(self.flag_op1_base_fp),
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
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset2 = if self.is_rel { Some(1) } else { None };

        // Check the instruction.
        let ([_, _, offset2], _) = ab.call(
            &CheckInstruction {
                const_offsets: [Some(0), Some(1), offset2],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Push fp.
        ab.set_in_memory(
            &self.memory,
            ap.clone(),
            Felt252Expr::from(vec![fp.clone()]),
        );

        // Push pc + instruction_size.
        ab.set_in_memory(
            &self.memory,
            ap.clone() + const_expr!(1),
            Felt252Expr::from(vec![(pc.clone() + const_expr!(1 + (self.is_rel as u32)))]),
        );

        // Update pc.
        let next_pc = if self.is_rel {
            pc.clone()
                + ab.call(
                    &ReadSmallFelt252 {
                        num_bits: FELT252_BITS_PER_WORD,
                        memory: self.memory.clone(),
                    },
                    pc + const_expr!(1),
                )
                .get_felt(0)
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

        [next_pc, ap.clone() + const_expr!(2), ap + const_expr!(2)]
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [
            ("is_rel".to_string(), self.is_rel.to_string()),
            (
                "flag_op1_base_fp".to_string(),
                self.flag_op1_base_fp.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for CallOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
