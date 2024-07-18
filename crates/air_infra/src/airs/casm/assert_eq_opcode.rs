use indexmap::IndexMap;

use super::check_instruction::*;
use super::common::*;
use super::read_small_felt252::*;

use crate::airs::casm::read_addr::ReadAddr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::FELT252_BITS_PER_WORD;

// Macros
use crate::const_expr;

/// The assert_eq opcode.
/// Implements the Cairo0 instructions:
/// - [ap/fp + offset0] = [ap/fp + offset2]
/// - [ap/fp + offset0] = [[ap/fp + offset1] + offset2]
/// - [ap/fp + offset0] = imm

#[derive(Clone, Debug)]
pub struct AssertEqOpcode {
    pub is_double_deref: bool,
    pub is_immediate: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AssertEqOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: if self.is_double_deref {
                None
            } else {
                Some(true) // Default is fp based
            },
            op1_imm: Some(self.is_immediate),
            op1_base_fp: if !self.is_double_deref && !self.is_immediate {
                None
            } else {
                Some(false)
            },
            op1_base_ap: if !self.is_double_deref && !self.is_immediate {
                None
            } else {
                Some(false)
            },
            res_add: Some(false),
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

impl AirFn for AssertEqOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        assert!(
            !(self.is_immediate && self.is_double_deref),
            "Double deref and immediate can't be set together"
        );

        // Create the constant offsets.
        let offsets = if self.is_immediate {
            [None, Some(offset_as_u16(-1)), Some(offset_as_u16(1))]
        } else if self.is_double_deref {
            [None, None, None]
        } else {
            [None, Some(offset_as_u16(-1)), None]
        };

        // Check the instruction.
        let ([offset0, offset1, offset2], flags) = ab.call(
            &CheckInstruction {
                const_offsets: offsets,
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Read the non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP].as_felt();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP].as_felt();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP].as_felt();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP].as_felt();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1].as_felt();

        // Fetch dst
        let read_12bits_felt = ReadSmallFelt252 {
            num_bits: FELT252_BITS_PER_WORD,
            memory: self.memory.clone(),
        };
        let mem_dst_base = flag_dst_base_fp.clone() * fp.clone()
            + (const_expr!(1) - flag_dst_base_fp) * ap.clone();
        let dst = ab.call(&read_12bits_felt, mem_dst_base + offset0);

        // Find mem1_base
        let mem1_base = if self.is_double_deref {
            let mem0_base = flag_op0_base_fp.clone() * fp.clone()
                + (const_expr!(1) - flag_op0_base_fp) * ap.clone();
            ab.call(
                &ReadAddr {
                    memory: self.memory.clone(),
                },
                mem0_base + offset1,
            )
        } else if self.is_immediate {
            pc.clone()
        } else {
            ab.constrain(flag_op1_base_fp.clone() + flag_op1_base_ap.clone() - const_expr!(1));
            flag_op1_base_fp * fp.clone() + flag_op1_base_ap * ap.clone()
        };

        // Assert that dst == op1
        ab.set_in_memory(&self.memory, mem1_base + offset2, dst);

        // Calculate the next ap
        let next_ap = (const_expr!(1) - flag_ap_update_add_1.clone()) * ap.clone()
            + flag_ap_update_add_1 * (ap + const_expr!(1));

        // Calculate the next pc
        let next_pc = if self.is_immediate {
            pc + const_expr!(2)
        } else {
            pc + const_expr!(1)
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> IndexMap<std::string::String, std::string::String> {
        [(
            "is_double_deref".to_string(),
            self.is_double_deref.to_string(),
        )]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for AssertEqOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
