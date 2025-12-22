use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::pedersen::PEDERSEN_TABLE_N_COLUMNS;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

#[cfg(test)]
use super::utils::*;
use crate::airs::casm::const_tables::seq::*;
#[cfg(test)]
use crate::const_felt252_expr_from_felt252;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;

// A table with 2**23 rows, each containing a point on the Pedersen elliptic curve.
// The table is divided into 2 sections:
// 1a. First 13 blocks of 2**18 rows: Row k of block b contains -P_shift + 2**(18*b) * k * P_0
// 1b. The 14th block of 2**18 rows: Row k + (l << 14) contains
//       -P_shift + 2**(18*13) * k * P_0 + l * P_1
// 2a. Next 13 blocks of 2**18 rows: Row k of block b contains -P_shift + 2**(18*b) * k * P_2
// 2b. The last block of 2**18 rows: Row k + (l << 14) contains
//       -P_shift + 2**(18*13) * k * P_2 + l * P_3
#[derive(Clone, Debug, Default)]
pub struct PedersenPoints {}

// TODO: Take from stwo-cairo-common.
pub const BITS_PER_WINDOW: usize = 18;
pub const NUM_WINDOWS: usize = 252usize.div_ceil(BITS_PER_WINDOW);
pub const ROWS_PER_WINDOW: usize = 1 << BITS_PER_WINDOW;
pub const BITS_IN_LAST_WINDOW: usize = 14;
pub const ROWS_IN_LAST_WINDOW: usize = 1 << BITS_IN_LAST_WINDOW;
pub const P_0_SECTION_START: usize = 0;
pub const P_2_SECTION_START: usize = P_0_SECTION_START + NUM_WINDOWS * ROWS_PER_WINDOW;
#[cfg(test)]
const TABLE_END: usize = P_2_SECTION_START + NUM_WINDOWS * ROWS_PER_WINDOW;

#[cfg(test)]
fn compute_section_row(
    row_in_section: usize,
    base_point: &CurvePoint,
    high_base_point: &CurvePoint,
) -> CurvePoint {
    assert!(row_in_section < NUM_WINDOWS * ROWS_PER_WINDOW);
    let block_num = row_in_section / ROWS_PER_WINDOW;
    let row_in_block = if block_num < NUM_WINDOWS - 1 {
        row_in_section % ROWS_PER_WINDOW
    } else {
        row_in_section % ROWS_IN_LAST_WINDOW
    };
    let minus_p_shift = ec_neg(&P_SHIFT);
    let result = ec_add_mul(
        &minus_p_shift,
        &ec_shift(base_point, BITS_PER_WINDOW * block_num),
        row_in_block,
    );
    if block_num < NUM_WINDOWS - 1 {
        result
    } else {
        ec_add_mul(
            &result,
            high_base_point,
            (row_in_section % ROWS_PER_WINDOW) >> BITS_IN_LAST_WINDOW,
        )
    }
}

impl ExtTable for PedersenPoints {
    type T = [Felt252Expr; 2];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            let point = match row_number {
                P_0_SECTION_START..P_2_SECTION_START => {
                    let row_in_section = row_number - P_0_SECTION_START;
                    compute_section_row(row_in_section, &P_0, &P_1)
                }
                P_2_SECTION_START..TABLE_END => {
                    let row_in_section = row_number - P_2_SECTION_START;
                    compute_section_row(row_in_section, &P_2, &P_3)
                }
                _ => panic!("Access to row {} in PedersenPoints", row_number),
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
                Box::new(stwo_cairo_common::preprocessed_columns::pedersen::PedersenPoints::new(i))
                    as Box<dyn PreProcessedColumn>
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct PedersenPointsTable {}

impl AirFn for PedersenPointsTable {
    type ExtIn = SeqConstLen<23>;
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
        air_builder.call_external_table(&PedersenPoints {})
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
