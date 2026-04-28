#[cfg(test)]
use std::array::from_fn;

#[cfg(test)]
use air_infra::const_expr;
use air_infra::core::air_fn::AirBuilder;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::variables::ExtTable;
#[cfg(test)]
use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;
use stwo_cairo_common::preprocessed_columns::blake::N_BLAKE_SIGMA_COLS;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

/// A constant table for the Sigma message permutation for Blake.
/// There are 10 rows, where each row `i` contains the permutation of the message for round `i`.
/// Accessed by the RoundSigma component through an external column call.
#[derive(Debug, Default, Clone)]
pub struct BlakeSigma {}
impl ExtTable for BlakeSigma {
    type T = [FeltExpr; 16];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return from_fn(|i| const_expr!(BLAKE_SIGMA[row_number][i]));
        }

        Self::T::default()
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        (0..N_BLAKE_SIGMA_COLS)
            .map(|i| {
                Box::new(stwo_cairo_common::preprocessed_columns::blake::BlakeSigma::new(i))
                    as Box<dyn PreProcessedColumn>
            })
            .collect()
    }
}
