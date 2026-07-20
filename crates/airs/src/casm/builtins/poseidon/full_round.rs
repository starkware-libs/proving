use air_common::TraceType;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn, ChainRoundAirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::variables::{ChainIdVar, RoundNumVar};
use serde::Serialize;

use super::cube252::*;
use super::linear_combination::*;
use super::round_keys::*;

/// Computes and verifies full rounds of Poseidon in a chain lookup.
/// The inputs are all passed into Cube252, and therefore are range checked by this air.
/// The outputs are LinearCombination outputs, and thus are not range checked by this air, and must
/// be range checked elsewhere (typically in the next full round, or directly otherwise).
#[derive(Clone, Debug, Serialize)]
pub struct PoseidonFullRoundChain {}

impl AirFn for PoseidonFullRoundChain {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, [Felt252Width27Expr; 3]);
    type Out = (ChainIdVar, RoundNumVar, [Felt252Width27Expr; 3]);

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (chain, round, state): Self::In,
    ) -> Self::Out {
        let [x, y, z] = state.map(|a| air_builder.lookup_call(&Cube252 {}, (), a));
        let [key_x, key_y, key_z] =
            air_builder.lookup_call(&PoseidonRoundKeys {}, [round.clone()], ());
        let x_new = air_builder
            .call(&LinearCombination::new([3, 1, 1, 1]), [x.clone(), y.clone(), z.clone(), key_x]);
        let y_new = air_builder
            .call(&LinearCombination::new([1, -1, 1, 1]), [x.clone(), y.clone(), z.clone(), key_y]);
        let z_new = air_builder.call(&LinearCombination::new([1, 1, -2, 1]), [x, y, z, key_z]);

        let new_state = [x_new, y_new, z_new];
        (chain, round + const_expr!(1), new_state)
    }
}

impl ChainRoundAirFn<[Felt252Width27Expr; 3]> for PoseidonFullRoundChain {
    fn number_of_chains(&self) -> usize {
        2
    }
}
