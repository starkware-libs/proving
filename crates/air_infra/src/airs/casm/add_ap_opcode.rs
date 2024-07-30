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

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm
/// - ap += [fp/ap+offset2]
/// - ap += [[ap/fp + offset1] + offset2]
/// - ap += [fp/ap+offset2] +/* [fp/ap+offset1]
/// - ap += [fp/ap+offset2] +/* imm
/// - ap += [fp/ap+offset1] + imm

#[derive(Clone, Debug)]
pub struct AddAp {
    pub is_immediate: bool,
    pub is_double_deref: bool,
    pub is_binary: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for AddAp {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset1 = if self.is_double_deref || self.is_binary {
            None
        } else {
            Some(offset_as_u16(-1))
        };
        let offset2 = if self.is_immediate {
            Some(offset_as_u16(1))
        } else {
            None
        };

        // Create constant flags.
        let flags = Flags {
            dst_base_fp: Some(true),
            op0_base_fp: if self.is_double_deref || self.is_binary {
                None
            } else {
                Some(true)
            },
            op1_imm: Some(self.is_immediate),
            op1_base_fp: if self.is_double_deref || self.is_immediate {
                Some(false)
            } else {
                None
            },
            op1_base_ap: if self.is_double_deref || self.is_immediate {
                Some(false)
            } else {
                None
            },
            res_add: if self.is_binary { None } else { Some(false) },
            res_mul: if self.is_binary { None } else { Some(false) },
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(true),
            ap_update_add_1: Some(false),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        };

        // Check the instruction.
        let ([_, offset1, offset2], flags) = ab.call(
            &CheckInstruction {
                const_offsets: [Some(offset_as_u16(-1)), offset1, offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Read the non-constant flags.
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP].as_felt();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP].as_felt();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP].as_felt();
        let flag_res_add = flags[FLAG_RES_ADD].as_felt();
        let flag_res_mul = flags[FLAG_RES_MUL].as_felt();

        // Fetch op1
        let op1 = if self.is_double_deref {
            let mem0_base = flag_op0_base_fp.clone() * fp.clone()
                + (const_expr!(1) - flag_op0_base_fp.clone()) * ap.clone();
            let op0 = ab.call(
                &ReadAddr {
                    memory: self.memory.clone(),
                },
                mem0_base + offset1.clone(),
            );
            ab.call(
                &ReadSmallFelt252 {
                    num_bits: FELT252_BITS_PER_WORD,
                    memory: self.memory.clone(),
                },
                op0 + offset2,
            )
            .as_felts()[0]
                .clone()
        } else if self.is_immediate {
            ab.call(
                &ReadSmallFelt252 {
                    num_bits: FELT252_BITS_PER_WORD,
                    memory: self.memory.clone(),
                },
                pc.clone() + const_expr!(1),
            )
            .as_felts()[0]
                .clone()
        } else {
            ab.constrain(flag_op1_base_fp.clone() + flag_op1_base_ap.clone() - const_expr!(1));
            let mem1_base = flag_op1_base_fp * fp.clone() + (flag_op1_base_ap.clone()) * ap.clone();
            ab.call(
                &ReadSmallFelt252 {
                    num_bits: FELT252_BITS_PER_WORD,
                    memory: self.memory.clone(),
                },
                mem1_base + offset2,
            )
            .as_felts()[0]
                .clone()
        };

        // Calculate the next ap
        let next_ap = ap.clone()
            + if self.is_binary {
                ab.constrain(flag_res_add.clone() + flag_res_mul.clone() - const_expr!(1));
                let mem0_base = flag_op0_base_fp.clone() * fp.clone()
                    + (const_expr!(1) - flag_op0_base_fp) * ap;
                let op0 = ab
                    .call(
                        &ReadSmallFelt252 {
                            num_bits: FELT252_BITS_PER_WORD,
                            memory: self.memory.clone(),
                        },
                        mem0_base + offset1,
                    )
                    .as_felts()[0]
                    .clone();
                flag_res_add * (op0.clone() + op1.clone()) + flag_res_mul * (op0 * op1)
            } else {
                op1
            };

        // Calculate the next pc
        let next_pc = if self.is_immediate {
            pc + const_expr!(2)
        } else {
            pc + const_expr!(1)
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            ("is_immediate".to_string(), self.is_immediate.to_string()),
            (
                "is_double_deref".to_string(),
                self.is_double_deref.to_string(),
            ),
            ("is_binary".to_string(), self.is_binary.to_string()),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for AddAp {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
