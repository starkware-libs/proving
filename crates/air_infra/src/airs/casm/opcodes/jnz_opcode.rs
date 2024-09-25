use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::decode_instruction::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::airs::memory::felt252_id_memory_read_positive::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Macros
use crate::const_expr;

/// The jnz opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm if [ap + offset] != 0
/// - jump rel imm if [fp + offset] != 0

#[derive(Clone, Debug, InstDef)]
pub struct JnzOpcode {
    pub is_taken: bool,
    pub dst_base_fp: bool,
    pub ap_update_add_1: bool,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl JnzOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(self.dst_base_fp),
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
            ap_update_add_1: Some(self.ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JnzOpcode {
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, casm_state: Self::In) -> Self::Out {
        // Check the instruction.
        let ([offset_dst, _, _], _) = ab.call(
            &DecodeInstruction {
                const_offsets: [None, Some(-1), Some(1)],
                const_flags: self.get_flags(),
                memory: self.memory.clone(),
            },
            casm_state.pc.clone(),
        );

        // Fetch dst - the value upon which the jump is conditioned.
        let mem_dst_base = if self.dst_base_fp {
            casm_state.fp.clone()
        } else {
            casm_state.ap.clone()
        };
        let dst_key = mem_dst_base + offset_dst;

        let dst = ab
            .call(
                &ReadPositive {
                    num_bits: 252,
                    memory: self.memory.clone(),
                },
                dst_key,
            )
            .0
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

            casm_state.pc.clone() + self.memory.read_rel_imm(ab, casm_state.pc + const_expr!(1))
        } else {
            // constrain dst == 0
            // This is sound because in this case it is sufficient to make sure that dst is zero.
            // The sum of the parts of dst is zero iff dst is zero because they are too small to wrap around m31.
            ab.constrain(dst_sum);
            casm_state.pc + const_expr!(2)
        };

        // Calculate the next ap
        let next_ap = casm_state.ap + const_expr!(self.ap_update_add_1 as u32);

        CasmStateVar::new(next_pc, next_ap, casm_state.fp)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
