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
use crate::felt252_utils::felt252_packing27::*;

/// Computes and verifies one partial round of Poseidon.
/// The partial round state at round i is denoted by [z0_3, z1, z1_3, z2], where zj is the value
/// of the third state element at round i+j-2, and zj_3 is its cube. From these values we can
/// compute z2_3 by cubing and z3 by the linear relation
///   z3 = 8*z0_3 + 4*z1 + 6*z1_3 + 2*z2 - 2*z2_3 + key,
/// where key is some fixed linear combination of the appropriate round keys.
/// We thus obtain the next partial round state at round i+1, which is [z1_3, z2, z2_3, z3].
#[derive(Clone, Debug, Serialize)]
pub struct PoseidonPartialRound {}

impl AirFn for PoseidonPartialRound {
    type ExtIn = ();
    type In = ([Felt252Width27Expr; 4], Felt252Width27Expr);
    type Out = [Felt252Width27Expr; 2];

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("z0_3".to_string()),
            Some("z1".to_string()),
            Some("z1_3".to_string()),
            Some("z2".to_string()),
            Some("half_key".to_string()),
        ])
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        ([z0_3, z1, z1_3, z2], half_key): Self::In,
    ) -> Self::Out {
        let z2_3 = air_builder.lookup_call(&Cube252 {}, (), z2.clone());
        // Unfortunately, the sum of coefficients of the full linear combination for z3 is slightly
        // too large to be processed directly. Fortunately, almost all coefficients are even, and
        // the only odd one is of the key, which can be freely modified to be even by replacing the
        // key used with half its value.
        // We thus first compute half of z3 using half-coefficients (which does fit in a single
        // lincomb), and then compute z3 by doubling half_z3, taking advantage of the fact that
        // doubling is a particularly cheap (i.e. lookup-free) type of linear combination.
        let half_z3 = air_builder.call(
            &LinearCombination::new([4, 2, 3, 1, -1, 1]),
            [z0_3, z1, z1_3, z2, z2_3.clone(), half_key],
        );
        // The intermediary value half_z3, unlike the partial round state elements, is not the input
        // or output of any Cube252, and thus needs to be directly range checked.
        air_builder.lookup_call(&RangeCheck252Width27 {}, (), half_z3.clone());
        let z3 = air_builder.call(&LinearCombination::new([2]), [half_z3]);

        [z2_3, z3]
    }
}

/// Computes and verifies three partial rounds of Poseidon at a time, in a chain lookup.
#[derive(Clone, Debug, Serialize)]
pub struct Poseidon3PartialRoundsChain {}

impl AirFn for Poseidon3PartialRoundsChain {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, [Felt252Width27Expr; 4]);
    type Out = (ChainIdVar, RoundNumVar, [Felt252Width27Expr; 4]);

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (chain, round, mut state): Self::In,
    ) -> Self::Out {
        let keys = air_builder.lookup_call(&PoseidonRoundKeys {}, [round.clone()], ());
        for k in keys {
            let [s2, s3] = air_builder.call(&PoseidonPartialRound {}, (state.clone(), k));
            state = [state[2].clone(), state[3].clone(), s2, s3];
        }

        (chain, round + const_expr!(1), state)
    }
}

impl ChainRoundAirFn<[Felt252Width27Expr; 4]> for Poseidon3PartialRoundsChain {
    fn number_of_chains(&self) -> usize {
        1
    }
}
