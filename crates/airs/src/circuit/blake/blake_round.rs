use air_common::TraceType;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn, ChainRoundAirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::{ChainIdVar, RoundNumVar};
use serde::Serialize;

use super::blake_message::*;
use crate::casm::opcodes::blake::g::*;
use crate::casm::opcodes::blake::round_sigma::*;

// Each row `i` contains the state's indices sent to the function `G` in the `i`-th call for each
// round.
pub const G_STATE_INDICES: [[usize; 4]; 8] = [
    [0, 4, 8, 12],
    [1, 5, 9, 13],
    [2, 6, 10, 14],
    [3, 7, 11, 15],
    [0, 5, 10, 15],
    [1, 6, 11, 12],
    [2, 7, 8, 13],
    [3, 4, 9, 14],
];

#[derive(Debug, Serialize)]
pub struct CircuitBlakeRound {
    #[serde(skip)]
    pub message: [UInt32Expr; 16],
}

impl AirFn for CircuitBlakeRound {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, ([UInt32Expr; 16], FeltExpr));
    type Out = (ChainIdVar, RoundNumVar, ([UInt32Expr; 16], FeltExpr));

    fn call(
        &self,
        ab: &mut AirBuilder,
        _: (),
        (chain, rnd, (mut state, message_id)): Self::In,
    ) -> Self::Out {
        // Read current message permutation (sigma) according to the round.
        let curr_sigma = ab.lookup_call(&BlakeRoundSigma {}, [rnd.clone()], ());

        // Read the current message according to the permutation.
        let blake_message = BlakeMessage { message: self.message.clone() };
        let mut current_message = vec![];
        for i in curr_sigma.into_iter() {
            let message_limbi = ab.lookup_call(&blake_message, (), [message_id.clone(), i]);
            current_message.push(message_limbi);
        }

        // Apply the G function to the state.
        for (row_index, &[i0, i1, i2, i3]) in G_STATE_INDICES.iter().enumerate() {
            [state[i0], state[i1], state[i2], state[i3]] = ab.lookup_call(
                &BlakeG {},
                (),
                [
                    state[i0].clone(),
                    state[i1].clone(),
                    state[i2].clone(),
                    state[i3].clone(),
                    current_message[row_index * 2].clone(),
                    current_message[row_index * 2 + 1].clone(),
                ],
            );
        }

        (chain, rnd + const_expr!(1), (state, message_id))
    }

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }
}

impl ChainRoundAirFn<([UInt32Expr; 16], FeltExpr)> for CircuitBlakeRound {
    fn number_of_chains(&self) -> usize {
        1
    }
}
