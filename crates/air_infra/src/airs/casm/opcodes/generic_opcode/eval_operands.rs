use inst_def::InstDef;

use super::generic_opcode::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::airs::felt252_id_memory::read_positive::*;
use crate::airs::felt252_utils::add252::*;
use crate::airs::felt252_utils::cond_as_small::*;
use crate::airs::felt252_utils::mul252::*;
//  Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

// Reads and verifies op0, op1 and dst from the memory.
// Calculates res and adds the relevant constraints.
#[derive(Clone, Debug, InstDef)]
pub struct EvalOperands {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for EvalOperands {
    type In = (CasmStateVar, [FeltExpr; GENERIC_FLAGS_SIZE], [FeltExpr; 3]);
    type Out = [Felt252Expr; 4];

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        (casm_state, flags, [offset0, offset1, offset2]): Self::In,
    ) -> Self::Out {
        // Read 252 bits since we don't know for what purpose is the reading
        let read_felt_252 = ReadPositive {
            num_bits: 252,
            memory: self.memory.clone(),
        };

        // Read dst
        let dst_src = flags[FLAG_DST_BASE_FP_INDEX].clone() * casm_state.fp.value.clone()
            + (const_expr!(1) - flags[FLAG_DST_BASE_FP_INDEX].clone())
                * casm_state.ap.value.clone();
        let (dst, _) = air_builder.call(&read_felt_252, CasmAddress::new(dst_src + offset0, "dst"));

        // Read op0
        let op0_src = flags[FLAG_OP0_BASE_FP_INDEX].clone() * casm_state.fp.value.clone()
            + (const_expr!(1) - flags[FLAG_OP0_BASE_FP_INDEX].clone())
                * casm_state.ap.value.clone();
        let (op0, _) = air_builder.call(&read_felt_252, CasmAddress::new(op0_src + offset1, "op0"));

        // Read op1
        let op0_as_addr = air_builder.call(
            &CondFelt252AsAddr {},
            (op0.clone(), flags[FLAG_OP1_BASE_OP0_INDEX].clone()),
        );

        let op1_src = flags[FLAG_OP1_BASE_FP_INDEX].clone() * casm_state.fp.value.clone()
            + flags[FLAG_OP1_BASE_AP_INDEX].clone() * casm_state.ap.value.clone()
            + flags[FLAG_OP1_IMM_INDEX].clone() * casm_state.pc.value.clone()
            + flags[FLAG_OP1_BASE_OP0_INDEX].clone() * op0_as_addr.value;
        let (op1, _) = air_builder.call(&read_felt_252, CasmAddress::new(op1_src + offset2, "op1"));

        let sum = air_builder.call(&Add252 {}, [op0.clone(), op1.clone()]);
        let prod = air_builder.call(&Mul252 {}, [op0.clone(), op1.clone()]);
        let mut res = air_builder.let_for_deduction(
            Felt252Expr::from(flags[FLAG_RES_OP1_INDEX].clone()) * op1.clone()
                + Felt252Expr::from(flags[FLAG_RES_MUL_INDEX].clone()) * prod.clone()
                + Felt252Expr::from(flags[FLAG_RES_ADD_INDEX].clone()) * sum.clone(),
            "res",
        );

        let res_constrained = air_builder.let_for_constraint(
            const_expr!(1) - flags[FLAG_PC_UPDATE_JNZ_INDEX].clone(),
            "res_constrained",
        );
        for (i, (res_felt, (op1_felt, (sum_felt, prod_felt)))) in res
            .as_felts_mut()
            .into_iter()
            .zip(
                op1.as_felts()
                    .iter()
                    .zip(sum.as_felts().iter().zip(prod.as_felts())),
            )
            .enumerate()
        {
            air_builder.deduce(res_felt, &format!("res_limb_{}", i));
            air_builder.constrain(
                (res_constrained.clone())
                    * (flags[FLAG_RES_OP1_INDEX].clone() * (res_felt.clone() - op1_felt.clone())
                        + flags[FLAG_RES_ADD_INDEX].clone()
                            * (res_felt.clone() - sum_felt.clone())
                        + flags[FLAG_RES_MUL_INDEX].clone()
                            * (res_felt.clone() - prod_felt.clone())),
                "",
            );
        }
        [dst, op0, op1, res]
    }
}
