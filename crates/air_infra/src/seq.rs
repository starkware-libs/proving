use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

use crate::casm_state::*;
#[cfg(any(test, feature = "test"))]
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::felt252_id_memory::memory::*;
use crate::range_check::*;

const STWO_COMPONENT_TYPE_SEQ: &str = "Seq";

/// A constant sequential column - row <i> contains the value <i>
#[derive(Debug, Default, Clone)]
pub struct Seq {}

impl ExtTable for Seq {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec![STWO_COMPONENT_TYPE_SEQ.to_owned()]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(any(test, feature = "test"))]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return const_expr!(row_number);
        }

        Self::T::default()
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct SeqConstLen<const L: usize> {}

impl<const L: usize> ExtTable for SeqConstLen<L> {
    type T = [FeltExpr; 1];

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![Box::new(stwo_cairo_common::preprocessed_columns::preprocessed_trace::Seq::new(
            L.try_into().unwrap(),
        ))]
    }
}

impl<const L: usize> RangeCheckSize for SeqConstLen<L> {
    fn bits() -> &'static [u16] {
        &[L as u16]
    }
}

#[derive(Debug, Default, Clone)]
pub struct SeqAddr {}

impl ExtTable for SeqAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec![STWO_COMPONENT_TYPE_SEQ.to_owned()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct SeqId {}

impl ExtTable for SeqId {
    type T = CasmId;

    fn column_ids() -> Vec<String> {
        vec![STWO_COMPONENT_TYPE_SEQ.to_owned()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}
