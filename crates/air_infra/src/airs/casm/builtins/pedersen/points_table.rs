use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

#[cfg(test)]
use super::utils::*;
use crate::airs::casm::const_tables::seq::*;
#[cfg(test)]
use crate::const_felt252_expr_from_felt252;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_PEDERSEN_POINTS: &str = "PedersenPoints";

// A table with 2**23 rows, each containing a point on the Pedersen elliptic curve.
// The table is divided into 3 sections:
// 1. First 14 blocks of 2 ** 18 rows: Row k of block b contains -P_shift + 2**(18*b) * k * P_0
// 2. Next 14 blocks of 2 ** 18 rows: Row k of block b contains -P_shift + 2**(18*b) * k * P_2
// 3. Next 256 rows: Row k + (16 * l) contains 29 * P_shift + k * P_1 + l * P_3
#[derive(Clone, Debug, Default)]
pub struct PedersenPoints {}

// TODO: Take from stwo-cairo-common.
pub const BITS_PER_WINDOW: usize = 18;
pub const NUM_WINDOWS: usize = 252usize.div_ceil(BITS_PER_WINDOW);
pub const ROWS_PER_WINDOW: usize = 1 << BITS_PER_WINDOW;
pub const P_0_SECTION_START: usize = 0;
pub const P_2_SECTION_START: usize = P_0_SECTION_START + NUM_WINDOWS * ROWS_PER_WINDOW;
pub const P_13_SECTION_START: usize = P_2_SECTION_START + NUM_WINDOWS * ROWS_PER_WINDOW;
#[cfg(test)]
const TABLE_END: usize = P_13_SECTION_START + 16 * 16;

#[cfg(test)]
fn compute_section_row(row_in_section: usize, base_point: &CurvePoint) -> CurvePoint {
    assert!(row_in_section < NUM_WINDOWS * ROWS_PER_WINDOW);
    let block_num = row_in_section / ROWS_PER_WINDOW;
    let row_in_block = row_in_section % ROWS_PER_WINDOW;
    let minus_p_shift = ec_neg(&P_SHIFT);
    ec_add_mul(
        &minus_p_shift,
        &ec_shift(base_point, BITS_PER_WINDOW * block_num),
        row_in_block,
    )
}

impl ExtTable for PedersenPoints {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_PEDERSEN_POINTS;

    type T = [Felt252Expr; 2];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            let point = match row_number {
                P_0_SECTION_START..P_2_SECTION_START => {
                    let row_in_section = row_number - P_0_SECTION_START;
                    compute_section_row(row_in_section, &P_0)
                }
                P_2_SECTION_START..P_13_SECTION_START => {
                    let row_in_section = row_number - P_2_SECTION_START;
                    compute_section_row(row_in_section, &P_2)
                }
                P_13_SECTION_START..TABLE_END => {
                    let row_in_section = row_number - P_13_SECTION_START;
                    let (row_low, row_high) = (row_in_section & 0xf, row_in_section >> 4);
                    ec_add_mul(
                        &ec_add_mul(&ec_mul(&P_SHIFT, 2 * NUM_WINDOWS + 1), &P_1, row_low),
                        &P_3,
                        row_high,
                    )
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

    fn preprocessed_column() -> Option<Box<dyn PreProcessedColumn>> {
        Some(Box::new(
            stwo_cairo_common::preprocessed_columns::pedersen::PedersenPoints::new(0),
        ))
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

    fn deduce_output(&self) -> Option<String> {
        // TODO: Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}
