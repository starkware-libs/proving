use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

use super::generic_opcode::*;
use crate::casm::common::*;
use crate::felt252_utils::add252::*;
use crate::felt252_utils::cond_as_small::*;
use crate::felt252_utils::mul252::*;

// Reads and verifies op0, op1 and dst from the memory.
// Calculates res and adds the relevant constraints.
#[derive(Clone, Debug, Serialize)]
pub struct EvalOperands {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for EvalOperands {
    type ExtIn = ();
    type In = (CasmStateVar, [FeltExpr; GENERIC_FLAGS_SIZE], [FeltExpr; 3]);
    type Out = [Felt252Expr; 4];

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        let mut result =
            vec![Some("pc".to_string()), Some("ap".to_string()), Some("fp".to_string())];
        for name in GENERIC_FLAG_NAMES.iter() {
            result.push(Some(name.to_string()))
        }
        result.push(Some("offset0".to_string()));
        result.push(Some("offset1".to_string()));
        result.push(Some("offset2".to_string()));
        Some(result)
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (casm_state, flags, [offset0, offset1, offset2]): Self::In,
    ) -> Self::Out {
        // Read 252 bits since we don't know for what purpose is the reading
        // Read dst
        let mut dst_src = flags[FLAG_DST_BASE_FP_INDEX].clone() * casm_state.fp().var
            + (const_expr!(1) - flags[FLAG_DST_BASE_FP_INDEX].clone()) * casm_state.ap().var;
        air_builder.assign(&mut dst_src, "dst_src");
        let dst = self.memory.read_felt252(air_builder, CasmAddress::new(dst_src + offset0, "dst"));

        // Read op0
        let mut op0_src = flags[FLAG_OP0_BASE_FP_INDEX].clone() * casm_state.fp().var
            + (const_expr!(1) - flags[FLAG_OP0_BASE_FP_INDEX].clone()) * casm_state.ap().var;
        air_builder.assign(&mut op0_src, "op0_src");
        let op0 = self.memory.read_felt252(air_builder, CasmAddress::new(op0_src + offset1, "op0"));

        // Read op1
        let op0_as_addr = air_builder
            .call(&CondFelt252AsAddr {}, (op0.clone(), flags[FLAG_OP1_BASE_OP0_INDEX].clone()));

        let mut op1_src = flags[FLAG_OP1_BASE_FP_INDEX].clone() * casm_state.fp().var
            + flags[FLAG_OP1_BASE_AP_INDEX].clone() * casm_state.ap().var
            + flags[FLAG_OP1_IMM_INDEX].clone() * casm_state.pc().var
            + flags[FLAG_OP1_BASE_OP0_INDEX].clone() * op0_as_addr.var;
        air_builder.assign(&mut op1_src, "op1_src");
        let op1 = self.memory.read_felt252(air_builder, CasmAddress::new(op1_src + offset2, "op1"));

        let sum = air_builder.call(&Add252 {}, [op0.clone(), op1.clone()]);
        let prod = air_builder.call(&Mul252 {}, [op0.clone(), op1.clone()]);
        let res = air_builder.deduce_air_var(
            Felt252Expr::from(flags[FLAG_RES_OP1_INDEX].clone()) * op1.clone()
                + Felt252Expr::from(flags[FLAG_RES_MUL_INDEX].clone()) * prod.clone()
                + Felt252Expr::from(flags[FLAG_RES_ADD_INDEX].clone()) * sum.clone(),
            "res",
        );

        let not_pc_update_jnz = air_builder.let_for_constraint(
            const_expr!(1) - flags[FLAG_PC_UPDATE_JNZ_INDEX].clone(),
            "not_pc_update_jnz",
        );
        for i in 0..FELT252_N_WORDS {
            air_builder.constrain(
                flags[FLAG_RES_ADD_INDEX].clone() * sum.get_felt(i)
                    + flags[FLAG_RES_MUL_INDEX].clone() * prod.get_felt(i)
                    + flags[FLAG_RES_OP1_INDEX].clone() * op1.get_felt(i)
                    - not_pc_update_jnz.clone() * res.get_felt(i),
                &format!("constrain limb {i} of res"),
            );
        }

        [dst, op0, op1, res]
    }
}
