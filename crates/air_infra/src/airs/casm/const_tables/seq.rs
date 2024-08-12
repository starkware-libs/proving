use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

#[cfg(test)]
use crate::expr;

#[derive(Debug)]
pub struct Seq {}

/// A constant sequential column - row <i> contains the value <i>
impl AirFn for Seq {
    type In = ();
    type Out = FeltExpr;

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            return expr!("seq", _air_builder.row_number());
        }
        FeltExpr::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}
