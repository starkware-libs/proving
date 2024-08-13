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
        return FeltExpr::Var(VarExpr::new(
            "seq".to_string(),
            _air_builder.row_number().map(|x| Felt::from(x as u32)),
            false,
            true,
        ));

        #[cfg(not(test))]
        FeltExpr::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}
