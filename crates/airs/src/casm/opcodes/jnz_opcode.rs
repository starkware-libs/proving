use std::collections::BTreeSet;

use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::verify::MemVerify;
use air_infra::{const_expr, const_felt252_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::P_FELTS;

use super::super::decode_instruction::decode_inst::*;
use crate::casm::common::*;

/// The jnz opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm if [ap + offset] != 0
/// - jump rel imm if [fp + offset] != 0

#[derive(Clone, Debug, Serialize)]
pub struct JnzOpcode {
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub taken: bool,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl JnzOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
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
            ap_update_add_1: None,
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JnzOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Check the instruction.
        let ([offset_dst, _, _], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [None, Some(-1), Some(1)],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1: BTreeSet::new(),
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Read non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        // Fetch dst - the value upon which the jump is conditioned.
        let mem_dst_base = ab.assign(
            &mut (flag_dst_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap().var),
            "mem_dst_base",
        );

        // Calculate the next pc
        let next_pc = if self.taken {
            // constrain dst != 0
            // This is sound because in this case it is sufficient to make sure that dst is not zero
            // or P (since 2P>2^252). The sum of the parts of dst is not zero iff dst is
            // not zero because they are too small to wrap around m31. Hence dst is not
            // zero iff the sum has an inverse modulo m31. We take a different sum where
            // the parts i where P is not zero are replaced by (dst[i]-P[i])^2. This sum
            // still can't wrap around m31 and is zero iff dst is P. Hence dst is not P
            // iff this sum has an inverse modulo m31.
            let dst = self
                .memory
                .read_felt252(ab, CasmAddress::new(mem_dst_base + offset_dst, "dst"))
                .as_felts();

            let (p_zero_zip_dst, p_nonzero_zip_dst) =
                P_FELTS.iter().zip(dst).partition::<Vec<_>, _>(|&(&p_i, _)| p_i == 0);

            // dst_sum_p_zero is the sum of dst[i] for i where P_FELTS[i] == 0
            let dst_sum_p_zero: FeltExpr =
                ab.let_(p_zero_zip_dst.into_iter().map(|(_, dst_i)| dst_i).sum(), "dst_sum_p_zero");

            let dst_sum = dst_sum_p_zero.clone()
                + p_nonzero_zip_dst.clone().into_iter().map(|(_, dst_i)| dst_i).sum();

            let dst_sum_inv = ab.deduce(&mut dst_sum.clone().inverse(), "dst_sum_inv");
            ab.constrain(dst_sum * dst_sum_inv - const_expr!(1), "dst doesn't equal 0");

            let dst_sum_squares = dst_sum_p_zero
                + p_nonzero_zip_dst
                    .into_iter()
                    .map(|(&p_i, dst_i)| {
                        let x = ab.let_(dst_i - const_expr!(p_i), "diff_from_p");
                        x.clone() * x
                    })
                    .sum();

            let dst_sum_squares_inv =
                ab.deduce(&mut dst_sum_squares.clone().inverse(), "dst_sum_squares_inv");
            ab.constrain(
                dst_sum_squares * dst_sum_squares_inv - const_expr!(1),
                "dst doesn't equal P",
            );

            casm_state.pc().var
                + self.memory.read_rel_imm(
                    ab,
                    CasmAddress::new(casm_state.pc().var + const_expr!(1), "next_pc"),
                )
        } else {
            // constrain dst == 0
            ab.call(
                &MemVerify { memory: self.memory.clone() },
                (CasmAddress::new(mem_dst_base + offset_dst, "dst"), const_felt252_expr!(0)),
            );
            casm_state.pc().var + const_expr!(2)
        };

        // Calculate the next ap
        let next_ap = casm_state.ap().var + ap_update_add_1;

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }

    fn name(&self) -> String {
        format!("jnz_opcode_{}", if self.taken { "taken" } else { "non_taken" })
    }
}
