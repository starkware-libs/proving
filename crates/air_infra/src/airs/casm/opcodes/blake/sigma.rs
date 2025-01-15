// Macros
#[cfg(test)]
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_BLAKE_SIGMA: &str = "BlakeSigma";

pub const SIGMA: [[u32; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// A constant table for the Sigma message permutation for Blake.
/// There are 10 rows, where each row `i` contains the permutation of the message for round `i`.
/// Accessed by the RoundSigma component through an external column call.
#[derive(Debug, Default, Clone)]
pub struct Sigma {}
impl ExtTable for Sigma {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_BLAKE_SIGMA;
    type T = [FeltExpr; 16];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return SIGMA[row_number]
                .iter()
                .map(|v| const_expr!(*v))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
        }

        Self::T::default()
    }
}
