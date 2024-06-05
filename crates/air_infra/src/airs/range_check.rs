use crate::core::air_fn::{AirFn, TraceType};
use crate::core::expressions::felt_expr::FeltExpr;

// TODO: Import from stwo once it is implemented there
const STWO_COMPONENT_TYPE_RANGE_CHECK_16: &str = "RangeCheck16";

#[derive(Debug)]
pub struct RangeCheck16 {}

impl AirFn for RangeCheck16 {
    type In = FeltExpr;

    type Out = ();

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_RANGE_CHECK_16.to_string()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }

    fn input_in_trace(&self) -> bool {
        true
    }

    fn call(
        &self,
        _air_builder: &mut crate::core::air_fn::AirBuilder,
        _input: Self::In,
    ) -> Self::Out {
        // TODO: In run mode, assert input < 2**16
        Self::Out::default()
    }
}
