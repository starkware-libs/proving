use std::array::from_fn;

use air_infra::casm_state::CasmAddress;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::felt252_id_memory::memory::ADDRESS_BITS;
use air_infra::felt252_id_memory::read_positive::CondRangeCheck2;
use air_infra::felt252_id_memory::read_small::{DecodeSmallSign, LIMBS_IN_SMALL, small_to_rel_imm};
use air_infra::utils::felt252_to_m31;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_N_WORDS;

// Receives a felt252 that has been written to the trace and adds constraints to verify that
// it is a valid address, i.e., between 0 and 2^29 - 1, if the given condition holds.
// Return the felt252 as an address.
#[derive(Debug, Serialize)]
pub struct CondFelt252AsAddr {}

impl AirFn for CondFelt252AsAddr {
    type ExtIn = ();
    type In = (Felt252Expr, FeltExpr);
    type Out = CasmAddress;

    fn call(&self, ab: &mut AirBuilder, _: (), (value, condition): Self::In) -> Self::Out {
        let high_limbs_sum =
            ((LIMBS_IN_SMALL + 1)..FELT252_N_WORDS).map(|i| value.get_felt(i)).sum();
        ab.constrain(
            condition.clone() * high_limbs_sum,
            "When the condition holds, the high limbs must be zero for an address",
        );

        ab.call(&CondRangeCheck2 {}, [value.get_felt(LIMBS_IN_SMALL), condition]);
        CasmAddress::new(felt252_to_m31(value, ADDRESS_BITS), "")
    }
}

// Receives a felt252 that has been written to the trace and adds constraints to verify that
// it is a valid relative immediate, i.e., between [-2^29 - 1, 2^29 - 1], if the given condition
// holds. The condition must be linear wrt the trace.
// Returns the felt252 as a relative immediate.
#[derive(Debug, Serialize)]
pub struct CondFelt252AsRelImm {}

impl AirFn for CondFelt252AsRelImm {
    type ExtIn = ();
    type In = (Felt252Expr, FeltExpr);
    type Out = FeltExpr;

    fn call(&self, ab: &mut AirBuilder, _: (), (value, condition): Self::In) -> Self::Out {
        // Compute the four values needed to construct the relative immediate other then the
        // low-limbs value.
        let [msb, mid_limbs_set, limb3_7_high_bits, limbs4_to_20, limb21, limb27] =
            ab.call(&DecodeSmallSign {}, value.clone());

        // Constrain the remainder bits.
        let remainder_bits =
            ab.let_(value.get_felt(LIMBS_IN_SMALL) - limb3_7_high_bits.clone(), "remainder_bits");
        ab.call(&CondRangeCheck2 {}, [remainder_bits.clone(), condition.clone()]);

        // Constrain limbs 4-20.
        let limbs_4_to_20_sum: FeltExpr = (4..=20).map(|i| value.get_felt(i)).sum();
        ab.constrain(
            condition.clone() * (limbs_4_to_20_sum - limbs4_to_20.clone() * const_expr!(17)),
            "When the condition holds, limbs 4-20 must be zero or 0x1ff",
        );

        // Constrain limb 21.
        ab.constrain(
            condition.clone() * (value.get_felt(21) - limb21.clone()),
            "When the condition holds, limb 21 must be 0x0, 0x88 or 0x87",
        );

        // Constrain limbs 22-26.
        let limbs_22_to_26_sum = (22..=26).map(|i| value.get_felt(i)).sum();
        ab.constrain(
            condition.clone() * limbs_22_to_26_sum,
            "When the condition holds, limbs 22-26 must be zero",
        );

        // Constrain limb 27.
        ab.constrain(
            condition.clone() * (value.get_felt(27) - limb27.clone()),
            "When the condition holds, limb 27 must be 0x0 or 0x100",
        );

        // Return the rel imm value
        let low_limbs_value: [FeltExpr; LIMBS_IN_SMALL] = from_fn(|i| value.get_felt(i));
        small_to_rel_imm(low_limbs_value, remainder_bits, msb, mid_limbs_set)
    }
}
