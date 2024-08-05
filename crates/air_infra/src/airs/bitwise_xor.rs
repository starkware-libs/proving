use crate::core::air_fn::AirBuilder;
use crate::core::air_fn::AirFn;
use crate::core::air_fn::TraceType;
#[cfg(test)]
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt_expr::FeltExpr;

const STWO_COMPONENT_TYPE_BITWISE_XOR: &str = "BitwiseXor";

#[derive(Debug)]
pub struct BitwiseXor {
    pub num_bits: usize,
}

// Asserts that the three felt expressions are in the correct range,
// and that their bitwise XOR is 0.
impl AirFn for BitwiseXor {
    type In = [FeltExpr; 3];
    type Out = ();

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_BITWISE_XOR.to_string()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }

    fn call(&self, _air_builder: &mut AirBuilder, [_a, _b, _c]: Self::In) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let a = _a.value().unwrap().0;
            let b = _b.value().unwrap().0;
            assert!(
                a < (1u32 << self.num_bits),
                "RangeCheck{} failed (input {})",
                self.num_bits,
                a
            );
            assert!(
                b < (1u32 << self.num_bits),
                "RangeCheck{} failed (input {})",
                self.num_bits,
                b
            );
            assert_eq!((a ^ b).to_string(), _c.calc());
        }
    }
}
