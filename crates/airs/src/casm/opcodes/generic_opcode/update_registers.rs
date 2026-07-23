use air_infra::casm_state::CasmStateVar;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AirVar;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::P_FELTS;

use super::generic_opcode::*;
use crate::casm::common::*;
use crate::casm::opcodes::add_ap_opcode::*;
use crate::felt252_utils::cond_as_small::*;

#[derive(Clone, Debug, Serialize)]
// Find the next pc, ap, fp and add constraints for jump not zero.
pub struct UpdateRegisters {}

impl AirFn for UpdateRegisters {
    type ExtIn = ();
    type In = (CasmStateVar, [FeltExpr; GENERIC_FLAGS_SIZE], [Felt252Expr; 3]);
    type Out = CasmStateVar;

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        let mut result =
            vec![Some("pc".to_string()), Some("ap".to_string()), Some("fp".to_string())];
        for name in GENERIC_FLAG_NAMES.iter() {
            result.push(Some(name.to_string()))
        }
        result.push(Some("dst".to_string()));
        result.push(Some("op1".to_string()));
        result.push(Some("res".to_string()));
        Some(result)
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("pc".to_string()), Some("ap".to_string()), Some("fp".to_string())])
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (casm_state, flags, [dst, op1, res]): Self::In,
    ) -> Self::Out {
        let dst_as_addr = air_builder
            .call(&CondFelt252AsAddr {}, (dst.clone(), flags[FLAG_OPCODE_RET_INDEX].clone()));

        let res_as_rel_imm = air_builder.call(
            &CondFelt252AsRelImm {},
            (
                res,
                flags[FLAG_PC_UPDATE_JUMP_REL_INDEX].clone()
                    + flags[FLAG_AP_UPDATE_ADD_INDEX].clone(),
            ),
        );

        // Handle jnz
        //  Assert dst!=p
        let dst_sum_squares: FeltExpr = dst
            .as_felts()
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                if P_FELTS[i] == 0 {
                    x
                } else {
                    let x = air_builder.let_(x - const_expr!(P_FELTS[i]), "diff_from_p");
                    x.clone() * x
                }
            })
            .sum();
        let sum_squares_inv =
            air_builder.deduce(&mut (dst_sum_squares.clone().inverse()), "dst_sum_squares_inv");
        air_builder.constrain(dst_sum_squares * sum_squares_inv - const_expr!(1), "dst_not_p");

        // Calculate npc for jnz
        let mut dst_sum: FeltExpr = dst.as_felts().clone().into_iter().sum();
        dst_sum = air_builder.let_(dst_sum, "dst_sum");

        let dst_is_zero =
            air_builder.let_for_deduction(dst_sum.clone().eq(const_expr!(0)), "dst_is_zero");
        // If dst_sum is 0, then sum_inv = 1
        let sum_inv = air_builder
            .deduce(&mut ((dst_sum.clone() + dst_is_zero.as_felt()).inverse()), "dst_sum_inv");

        // We use op1 as rel imm only if flags[FLAG_PC_UPDATE_JNZ_INDEX] is set and dst!= 0
        let op1_as_rel_imm_condition = air_builder.assign(
            &mut (flags[FLAG_PC_UPDATE_JNZ_INDEX].clone() * dst_sum.clone()),
            "op1_as_rel_imm_cond",
        );
        let op1_as_rel_imm =
            air_builder.call(&CondFelt252AsRelImm {}, (op1, op1_as_rel_imm_condition));

        // Next pc for jnz = pc + op1_as_rel_imm if dst is not zero, else pc + instruction_size
        let npc_jnz = air_builder.deduce(
            &mut (dst_is_zero.as_felt()
                * (casm_state.pc().var + flags[INSTRUCTION_SIZE_INDEX].clone())
                + (const_expr!(1) - dst_is_zero.as_felt())
                    * (casm_state.pc().var + op1_as_rel_imm.clone())),
            "next_pc_jnz",
        );
        air_builder.constrain(
            (npc_jnz.clone() - (casm_state.pc().var + op1_as_rel_imm.clone())) * dst_sum.clone(),
            "Constraint1 for conditional jump",
        );
        air_builder.constrain(
            (npc_jnz.clone() - (casm_state.pc().var + flags[INSTRUCTION_SIZE_INDEX].clone()))
                * (dst_sum.clone() * sum_inv.clone() - const_expr!(1)),
            "Constraint2 for conditional jump",
        );

        // Update pc
        let mut next_pc = flags[FLAG_PC_UPDATE_REGULAR_INDEX].clone()
            * (casm_state.pc().var + flags[INSTRUCTION_SIZE_INDEX].clone())
            + flags[FLAG_PC_UPDATE_JUMP_INDEX].clone() * res_as_rel_imm.clone()
            + flags[FLAG_PC_UPDATE_JUMP_REL_INDEX].clone()
                * (casm_state.pc().var + res_as_rel_imm.clone())
            + flags[FLAG_PC_UPDATE_JNZ_INDEX].clone() * npc_jnz;
        air_builder.assign(&mut next_pc, "next_pc");

        // Update ap
        let mut next_ap = casm_state.ap().var
            + flags[FLAG_AP_UPDATE_ADD_INDEX].clone() * res_as_rel_imm
            + flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone() * const_expr!(1)
            + flags[FLAG_OPCODE_CALL_INDEX].clone() * const_expr!(2);
        air_builder.assign(&mut next_ap, "next_ap");

        air_builder.call(&RangeCheck29 {}, next_ap.clone());

        // Update fp
        let mut next_fp = flags[FLAG_FP_UPDATE_REGULAR_INDEX].clone() * casm_state.fp().var
            + flags[FLAG_OPCODE_RET_INDEX].clone() * dst_as_addr.var
            + flags[FLAG_OPCODE_CALL_INDEX].clone() * (casm_state.ap().var + const_expr!(2));
        air_builder.assign(&mut next_fp, "next_fp");

        CasmStateVar::new(next_pc, next_ap, next_fp)
    }
}
