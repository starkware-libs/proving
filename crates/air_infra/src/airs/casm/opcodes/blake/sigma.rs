#[cfg(test)]
use std::array::from_fn;

#[cfg(test)]
use stwo_cairo_common::preprocessed_consts::blake::BLAKE_SIGMA;

// Macros
#[cfg(test)]
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_BLAKE_SIGMA: &str = "BlakeSigma";

/// A constant table for the Sigma message permutation for Blake.
/// There are 10 rows, where each row `i` contains the permutation of the message for round `i`.
/// Accessed by the RoundSigma component through an external column call.
#[derive(Debug, Default, Clone)]
pub struct BlakeSigma {}
impl ExtTable for BlakeSigma {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_BLAKE_SIGMA;
    type T = [FeltExpr; 16];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return from_fn(|i| const_expr!(BLAKE_SIGMA[row_number][i]));
        }

        Self::T::default()
    }
}
