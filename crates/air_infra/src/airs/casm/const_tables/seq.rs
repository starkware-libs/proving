use super::range_check::*;
use crate::airs::casm::casm_state::*;
#[cfg(test)]
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_SEQ: &str = "Seq";

/// A constant sequential column - row <i> contains the value <i>
#[derive(Debug, Default, Clone)]
pub struct Seq {}

impl ExtTable for Seq {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_SEQ;
    type T = FeltExpr;

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return const_expr!(row_number);
        }

        Self::T::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SeqConstLen<const L: usize> {}

impl<const L: usize> ExtTable for SeqConstLen<L> {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_SEQ;
    type T = [FeltExpr; 1];

    fn args() -> Vec<String> {
        vec![L.to_string()]
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
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_SEQ;
    type T = CasmAddress;
}
