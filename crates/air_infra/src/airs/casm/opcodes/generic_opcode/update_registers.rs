use inst_def::InstDef;
use prover_types::cpu::P_FELTS;

use super::generic_opcode::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
use crate::airs::felt252_utils::cond_as_small::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

#[derive(Clone, Debug, InstDef)]
// Find the next pc, ap, fp and add constraints for jump not zero.
pub struct UpdateRegisters {}

impl AirFn for UpdateRegisters {
    type ExtIn = ();
    type In = (
        CasmStateVar,
        [FeltExpr; GENERIC_FLAGS_SIZE],
        [Felt252Expr; 3],
    );
    type Out = CasmStateVar;

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (casm_state, flags, [dst, op1, res]): Self::In,
    ) -> Self::Out {
        let res_as_addr = air_builder.call(
            &CondFelt252AsAddr {},
            (res.clone(), flags[FLAG_PC_UPDATE_JUMP_INDEX].clone()),
        );

        let dst_as_addr = air_builder.call(
            &CondFelt252AsAddr {},
            (dst.clone(), flags[FLAG_OPCODE_RET_INDEX].clone()),
        );

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
        let dst_sum_squares = dst
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
            .fold(const_expr!(0), |acc, z| acc + z);
        let sum_squares_inv = air_builder.deduce(
            &mut (dst_sum_squares.clone().inverse()),
            "dst_sum_squares_inv",
        );
        air_builder.constrain(
            dst_sum_squares * sum_squares_inv - const_expr!(1),
            "dst_not_p",
        );

        // Calcualte npc for jnz
        let dst_sum = dst
            .as_felts()
            .clone()
            .into_iter()
            .fold(const_expr!(0), |acc, x| acc + x);
        let dst_is_zero =
            air_builder.let_for_deduction(dst_sum.clone().eq(const_expr!(0)), "dst_is_zero");
        // If dst_sum is 0, then sum_inv = 1
        let sum_inv = air_builder.deduce(
            &mut ((dst_sum.clone() + dst_is_zero.as_felt()).inverse()),
            "dst_sum_inv",
        );

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
        let next_pc = flags[FLAG_PC_UPDATE_REGULAR_INDEX].clone()
            * (casm_state.pc().var + flags[INSTRUCTION_SIZE_INDEX].clone())
            + flags[FLAG_PC_UPDATE_JUMP_INDEX].clone() * res_as_addr.var
            + flags[FLAG_PC_UPDATE_JUMP_REL_INDEX].clone()
                * (casm_state.pc().var + res_as_rel_imm.clone())
            + flags[FLAG_PC_UPDATE_JNZ_INDEX].clone() * npc_jnz;

        // Update ap
        let next_ap = casm_state.ap().var
            + flags[FLAG_AP_UPDATE_ADD_INDEX].clone() * res_as_rel_imm
            + flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone() * const_expr!(1)
            + flags[FLAG_OPCODE_CALL_INDEX].clone() * const_expr!(2);

        // Update fp
        let next_fp = flags[FLAG_FP_UPDATE_REGULAR_INDEX].clone() * casm_state.fp().var
            + flags[FLAG_OPCODE_RET_INDEX].clone() * dst_as_addr.var
            + flags[FLAG_OPCODE_CALL_INDEX].clone() * (casm_state.ap().var + const_expr!(2));

        CasmStateVar::new(next_pc, next_ap, next_fp)
    }
}
