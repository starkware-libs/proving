use crate::const_expr;
use crate::core::air_fn::AirFn;
use crate::core::air_fn::TraceType;
use crate::core::expressions::felt_expr::FeltExpr;
#[cfg(test)]
use crate::core::expressions::uint16_expr::UInt16Expr;

#[derive(Debug)]
pub struct BitwiseAnd {}

// This AirFn takes two felt expressions and returns their bitwise AND and XOR.
// Currently, implemented only for tests.
// Should be implemented as a lookup table in the future.
impl AirFn for BitwiseAnd {
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn name(&self) -> String {
        "BitwiseAnd".to_string()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }

    fn call(
        &self,
        _air_builder: &mut crate::core::air_fn::AirBuilder,
        [_a, _b]: Self::In,
    ) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            return (UInt16Expr::from(_a) & UInt16Expr::from(_b)).as_felt();
        }
        const_expr!(1)
    }
}
