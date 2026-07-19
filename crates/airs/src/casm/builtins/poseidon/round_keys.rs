use air_common::TraceType;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
#[cfg(test)]
use air_infra::core::variables::AsProverType;
use air_infra::seq::SeqConstLen;
use serde::Serialize;

use super::keys::*;

/// Lookup component into the table of round keys of Poseidon.
/// The input represents a round number:
///    0..4 correspond to the first 4 full rounds;
///    4..31 correspond to 27 triplets of partial rounds (6-87);
///    31..35 correspond to the last 4 full rounds;
/// The output consists of 30 constant columns, representing 3 Felt252s in Width27 form.
#[derive(Debug, Serialize)]
pub struct PoseidonRoundKeys {}

impl AirFn for PoseidonRoundKeys {
    type ExtIn = SeqConstLen<6>;
    type In = ();
    type Out = [Felt252Width27Expr; 3];

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        [_round_number]: [FeltExpr; 1],
        _: (),
    ) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_round_number.value().map(|v| v.0 as usize));
        air_builder.call_external_table(&Keys {})
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
