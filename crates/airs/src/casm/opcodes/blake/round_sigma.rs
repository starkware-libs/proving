use air_common::TraceType;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
#[cfg(test)]
use air_infra::core::variables::AsProverType;
use air_infra::seq::SeqConstLen;
use serde::Serialize;

use super::sigma::*;

/// Lookup component into the constant table Sigma.
/// The input is the const column seq of size 2**4, representing the round number of Blake 0-9 with
/// extra padding.
/// The output consists of constant columns with a width of 16, containing the message permutation
/// for the corresponding round.
#[derive(Debug, Serialize)]
pub struct BlakeRoundSigma {}

impl AirFn for BlakeRoundSigma {
    type ExtIn = SeqConstLen<4>;
    type In = ();
    type Out = [FeltExpr; 16];

    fn call(&self, air_builder: &mut AirBuilder, [_round]: [FeltExpr; 1], _: ()) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_round.value().map(|v| v.0 as usize));
        air_builder.call_external_table(&BlakeSigma {})
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
