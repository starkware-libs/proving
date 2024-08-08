use indexmap::IndexMap;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

use super::check_instruction::*;
use super::common::*;
use super::read_small_felt252::*;

// Macros
use crate::const_expr;

/// The jnz opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm if [ap + offset] != 0
/// - jump rel imm if [fp + offset] != 0

#[derive(Clone, Debug)]
pub struct JnzOpcode {
    pub is_taken: bool,
    pub flag_dst_base_fp: bool,
    pub flag_ap_update_add_1: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl JnzOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(self.flag_dst_base_fp),
            op0_base_fp: Some(true),
            op1_imm: Some(true),
            op1_base_fp: Some(false),
            op1_base_ap: Some(false),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(true),
            ap_update_add: Some(false),
            ap_update_add_1: Some(self.flag_ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JnzOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Check the instruction.
        let ([offset_dst, _, _], _) = ab.call(
            &CheckInstruction {
                const_offsets: [None, Some(offset_as_u16(-1)), Some(offset_as_u16(1))],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Fetch dst - the value upon which the jump is conditioned.
        let mem_dst_base = if self.flag_dst_base_fp {
            fp.clone()
        } else {
            ap.clone()
        };
        let dst_key = mem_dst_base + offset_dst;

        let dst = ab
            .call(
                &ReadSmallFelt252 {
                    num_bits: 252,
                    memory: self.memory.clone(),
                },
                dst_key,
            )
            .as_felts();

        // Calculate the next pc
        let dst_sum = dst
            .clone()
            .into_iter()
            .fold(const_expr!(0), |acc, x| acc + x);

        let next_pc = if self.is_taken {
            // constrain dst != 0
            // This is sound because in this case it is sufficient to make sure that dst is not zero or P (since 2P>2^252).
            // The sum of the parts of dst is not zero iff dst is not zero because they are too small to wrap around m31.
            // Hence dst is not zero iff the sum has an inverse modulo m31.
            // We take a different sum where the parts i where P is not zero are replaced by (dst[i]-P[i])^2.
            // This sum still can't wrap around m31 and is zero iff dst is P.
            // Hence dst is not P iff this sum has an inverse modulo m31.
            let res = ab.deduce(&mut (const_expr!(1) / dst_sum.clone()));
            ab.constrain(dst_sum * res - const_expr!(1));

            let dst_sum_squares = dst
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    if P_FELTS[i] == 0 {
                        x
                    } else {
                        let x = ab.let_(x - const_expr!(P_FELTS[i]));
                        x.clone() * x
                    }
                })
                .fold(const_expr!(0), |acc, z| acc + z);

            let res_squares = ab.deduce(&mut (const_expr!(1) / dst_sum_squares.clone()));
            ab.constrain(dst_sum_squares * res_squares - const_expr!(1));

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
            // constrain dst == 0
            // This is sound because in this case it is sufficient to make sure that dst is zero.
            // The sum of the parts of dst is zero iff dst is zero because they are too small to wrap around m31.
            ab.constrain(dst_sum);
            pc + const_expr!(2)
        };

        // Calculate the next ap
        let next_ap = if self.flag_ap_update_add_1 {
            ap + const_expr!(1)
        } else {
            ap
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [
            ("is_taken".to_string(), self.is_taken.to_string()),
            (
                "flag_dst_base_fp".to_string(),
                self.flag_dst_base_fp.to_string(),
            ),
            (
                "flag_ap_update_add_1".to_string(),
                self.flag_ap_update_add_1.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for JnzOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
