use air_common::TraceType;
#[cfg(test)]
use air_infra::const_felt252_expr_from_felt252;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252_expr::Felt252Expr;
#[cfg(test)]
use air_infra::core::variables::AsProverType;
use air_infra::core::variables::ExtTable;
use air_infra::seq::SeqConstLen;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::pedersen::PEDERSEN_TABLE_N_COLUMNS;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

#[cfg(test)]
use crate::casm::builtins::ec_utils::utils::*;

// TODO(Dan): Fix documentation.
// A table with 2**15 rows, each containing a point on the Pedersen elliptic curve.
// The table is divided into 2 sections:
// 1a. First 27 blocks of 2**9 rows: Row k of block b contains -P_shift + 2**(9*b) * k * P_0
// 1b. The 28th block of 2**9 rows: Row k + (l << 5) contains
//       -P_shift + 2**(9*27) * k * P_0 + l * P_1
// 2a. Next 27 blocks of 2**9 rows: Row k of block b contains -P_shift + 2**(9*b) * k * P_2
// 2b. The last block of 2**9 rows: Row k + (l << 5) contains
//       -P_shift + 2**(9*13) * k * P_2 + l * P_3
#[derive(Clone, Debug, Default)]
pub struct PedersenPoints<const WINDOW_BITS: usize> {
    pub window_bits: usize,
}

impl<const WINDOW_BITS: usize> PedersenPoints<WINDOW_BITS> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { window_bits: WINDOW_BITS }
    }
}

#[cfg(test)]
fn compute_section_row(
    window_bits: usize,
    row_in_section: usize,
    base_point: &CurvePoint,
    high_base_point: &CurvePoint,
) -> CurvePoint {
    let num_windows = 252usize.div_ceil(window_bits);
    let rows_per_window = 1 << window_bits;
    let bits_in_last_window = window_bits - 4;
    let rows_in_last_window = 1 << bits_in_last_window;

    assert!(row_in_section < num_windows * rows_per_window);
    let block_num = row_in_section / rows_per_window;
    let row_in_block = if block_num < num_windows - 1 {
        row_in_section % rows_per_window
    } else {
        row_in_section % rows_in_last_window
    };
    let minus_p_shift = ec_neg(&P_SHIFT);
    let result =
        ec_add_mul(&minus_p_shift, &ec_shift(base_point, window_bits * block_num), row_in_block);
    if block_num < num_windows - 1 {
        result
    } else {
        ec_add_mul(
            &result,
            high_base_point,
            (row_in_section % rows_per_window) >> bits_in_last_window,
        )
    }
}

impl<const WINDOW_BITS: usize> ExtTable for PedersenPoints<WINDOW_BITS> {
    type T = [Felt252Expr; 2];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let num_windows = 252usize.div_ceil(self.window_bits);
            let rows_per_window = 1 << self.window_bits;
            let p_0_section_start = 0;
            let p_2_section_start = p_0_section_start + num_windows * rows_per_window;
            let table_end: usize = p_2_section_start + num_windows * rows_per_window;

            let row_number = _air_builder.row_number().expect("Row number not set");
            let point = if p_0_section_start <= row_number && row_number < p_2_section_start {
                let row_in_section = row_number - p_0_section_start;
                compute_section_row(self.window_bits, row_in_section, &P_0, &P_1)
            } else if p_2_section_start <= row_number && row_number < table_end {
                let row_in_section = row_number - p_2_section_start;
                compute_section_row(self.window_bits, row_in_section, &P_2, &P_3)
            } else {
                panic!("Access to row {row_number} in PedersenPoints")
            };
            return [
                const_felt252_expr_from_felt252!(point.x),
                const_felt252_expr_from_felt252!(point.y),
            ];
        }

        Self::T::default()
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        (0..PEDERSEN_TABLE_N_COLUMNS)
            .map(|i| {
                Box::new(stwo_cairo_common::preprocessed_columns::pedersen::PedersenPoints::<
                    WINDOW_BITS,
                >::new(i)) as Box<dyn PreProcessedColumn>
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct PedersenPointsTable<const LOG_N_ROWS: usize> {
    pub window_bits: usize,
}

impl<const LOG_N_ROWS: usize> AirFn for PedersenPointsTable<LOG_N_ROWS> {
    type ExtIn = SeqConstLen<LOG_N_ROWS>;
    type In = ();
    type Out = [Felt252Expr; 2];

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _ext_input: <Self::ExtIn as ExtTable>::T,
        _input: Self::In,
    ) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_ext_input[0].value().map(|v| v.0 as usize));
        match self.window_bits {
            9 => air_builder.call_external_table(&PedersenPoints::<9>::new()),
            18 => air_builder.call_external_table(&PedersenPoints::<18>::new()),
            _ => panic!("Unsupported window_bits value {}", self.window_bits),
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
