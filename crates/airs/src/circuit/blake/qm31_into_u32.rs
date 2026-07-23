use std::slice::from_ref;

use air_common::UseOrYield;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::range_check::range_check;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;

use super::blake_message::*;
use crate::casm::opcodes::blake::blake_compress_opcode::BLAKE_NUM_ROUNDS;

#[derive(Debug, Serialize, Default)]
pub struct QM31IntoU32 {}

impl AirFn for QM31IntoU32 {
    type ExtIn = ();
    type In = ([FeltExpr; 16], FeltExpr);
    type Out = [UInt32Expr; 16];

    fn call(&self, ab: &mut AirBuilder, _: (), (message, message_id): Self::In) -> Self::Out {
        let mut res = vec![];
        let dummy_message = vec![const_u32_expr!(0); 16];
        ab.registry.add_entry(&BlakeMessage { message: dummy_message.try_into().unwrap() });

        for (i, limbi) in message.into_iter().enumerate() {
            let limbi_u32 = UInt32Expr::from(limbi.clone());

            // Range check low of limb.
            let mut limbi_low =
                ab.let_for_deduction(limbi_u32.clone() & const_u32_expr!(0xFFFF), "limbi_low");
            let limbi_low = ab.deduce(limbi_low.low_mut().as_felt_mut(), "limbi_low");
            range_check(ab, &[16], from_ref(&limbi_low));

            // TODO(AnatG): Change to 2 rcs 16.
            // Range check high of limb.
            let mut limbi_high =
                ab.let_for_deduction(limbi_u32.clone() >> const_u32_expr!(16), "limbi_high");
            let limbi_high = ab.deduce(limbi_high.low_mut().as_felt_mut(), "limbi_high");
            range_check(ab, &[15], from_ref(&limbi_high));

            // Make sure that if limbi is zero mod P then limbi_low and limbi_high are zero.
            // Without this constraint a zero limb can be represented as either (0, 0) or (0x7fff,
            // 0xffff) in the blake message.
            let is_zero = ab.let_for_deduction(const_expr!(0).eq(limbi.clone()), "limbi_is_zero");
            let inv = ab.deduce(
                &mut (const_expr!(1) * (is_zero.as_felt() + limbi.clone()).inverse()),
                "limbi_inv_or_one",
            );
            ab.constrain(
                (limbi.clone() * inv - const_expr!(1)) * limbi_low.clone(),
                "limbi is zero then limbi_low is zero",
            );

            // Reconstruct limbi from its low and high parts.
            ab.constrain(
                limbi - (limbi_low.clone() + (limbi_high.clone() * const_expr!(1 << 16))),
                &format!("limb {i} reconstruction"),
            );

            ab.add_lookup_term(
                "BlakeMessage",
                vec![message_id.clone(), const_expr!(i), limbi_low.clone(), limbi_high.clone()],
                UseOrYield::Yield,
                const_expr!(BLAKE_NUM_ROUNDS),
            );

            res.push(vec![limbi_low, limbi_high].into());
        }

        res.try_into().expect("Expected 16 elements in output")
    }
}
