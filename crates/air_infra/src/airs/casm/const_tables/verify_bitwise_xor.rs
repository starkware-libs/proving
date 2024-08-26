use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
#[cfg(test)]
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR: &str = "VerifyBitwiseXor";

#[derive(Debug)]
pub struct VerifyBitwiseXor {
    pub num_bits: usize,
}

// Asserts that the three felt expressions are in the correct range,
// and that their bitwise XOR is 0.
impl AirFn for VerifyBitwiseXor {
    type In = [FeltExpr; 3];
    type Out = ();

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR.to_string()
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
            assert!(
                (a ^ b).to_string() == _c.calc(),
                "The bitwise XOR of {:b} and {:b} is not {:b}",
                a,
                b,
                _c.value().unwrap().0
            );
        }
    }
}
