use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
#[cfg(test)]
use crate::core::expressions::var_expr::*;

#[cfg(test)]
use crate::core::Felt;

#[derive(Debug)]
pub struct Seq {}

/// A constant sequential column - row <i> contains the value <i>
impl AirFn for Seq {
    type In = ();
    type Out = FeltExpr;

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return FeltExpr::Var(VarExpr::new(
                self.name(),
                Some(Felt::from(row_number as u32)),
                true,
                true,
                None,
            ));
        }

        Self::Out::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}
