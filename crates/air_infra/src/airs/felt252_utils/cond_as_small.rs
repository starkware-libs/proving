use std::array::from_fn;

use inst_def::InstDef;

use crate::airs::memory::felt252_id_memory::*;
use crate::airs::memory::felt252_id_memory_read_small::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Receives a felt252 that has been written to the trace and adds constraints to verify that
// it is a valid address, i.e., between 0 and 2^27, if the given condition holds.
// Return the felt252 as an address.
#[derive(Debug, InstDef)]
pub struct CondFelt252AsAddr {}

impl AirFn for CondFelt252AsAddr {
    type In = (Felt252Expr, FeltExpr);
    type Out = FeltExpr;

    fn call(&self, ab: &mut AirBuilder, (value, condition): Self::In) -> Self::Out {
        for i in LIMBS_IN_M31..FELT252_N_WORDS {
            ab.constrain(condition.clone() * value.get_felt(i));
        }
        Felt252IdMemory::felt252_to_addr(value)
    }
}

// Receives a felt252 that has been written to the trace and adds constraints to verify that
// it is a valid relative immediate, i.e., between [-2^27, 2^27 - 1], if the given condition holds.
// Returns the felt252 as a relative immediate.
#[derive(Debug, InstDef)]
pub struct CondFelt252AsRelImm {}

impl AirFn for CondFelt252AsRelImm {
    type In = (Felt252Expr, FeltExpr);
    type Out = FeltExpr;

    fn call(&self, ab: &mut AirBuilder, (value, condition): Self::In) -> Self::Out {
        // Compute and deduce "case" bits: msb and mid_limbs_set
        let [msb, mid_limbs] = ab.call(&CondDecodeSmallSign {}, (value.clone(), condition.clone()));

        // Build the expected value limbs
        let low_limbs_value: [FeltExpr; LIMBS_IN_M31] = from_fn(|i| value.get_felt(i));
        let expected_value =
            small_to_felt252(low_limbs_value.clone(), msb.clone(), mid_limbs.clone());

        // Verify that the given value is relative-immediate.
        // No need to constrain the first LIMBS_IN_M31 limbs.
        for (i, expected_limb) in expected_value
            .as_felts()
            .into_iter()
            .enumerate()
            .skip(LIMBS_IN_M31)
        {
            ab.constrain(condition.clone() * (value.get_felt(i) - expected_limb));
        }

        // Return the rel imm value
        small_to_rel_imm(low_limbs_value, msb, mid_limbs)
    }
}
