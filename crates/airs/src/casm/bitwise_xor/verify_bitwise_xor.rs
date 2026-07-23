use std::fmt::Debug;
use std::marker::PhantomData;

use air_common::TraceType;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
#[cfg(test)]
use air_infra::core::variables::AirVar;
use air_infra::core::variables::ExtTable;
use air_infra::utils::get_relation_variant_names;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

use crate::new_verify_bitwise_xor;

pub trait VerifyBitwiseXorSize: ExtTable + Debug + Default {
    fn bits() -> u16;
}

pub fn verify_bitwise_xor(ab: &mut AirBuilder, bits: u16, input: [FeltExpr; 3], variant: usize) {
    if bits != 8 {
        assert!(variant == 0, "Only variant 0 is supported for bits other than 8");
    }

    match bits {
        4 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor_4_Const>::default(), input, ()),
        7 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor_7_Const>::default(), input, ()),
        8 => ab.lookup_call_variant(
            &VerifyBitwiseXor::<VerifyBitwiseXor_8_Const>::default(),
            input,
            (),
            variant,
        ),
        9 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor_9_Const>::default(), input, ()),
        12 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor_12_Const>::default(), input, ()),
        _ => panic!("Unsupported verify bitwise xor bits: {bits:?}"),
    }
}

new_verify_bitwise_xor!(4, VerifyBitwiseXor_4_Const);
new_verify_bitwise_xor!(7, VerifyBitwiseXor_7_Const);
new_verify_bitwise_xor!(8, VerifyBitwiseXor_8_Const);
new_verify_bitwise_xor!(9, VerifyBitwiseXor_9_Const);
new_verify_bitwise_xor!(12, VerifyBitwiseXor_12_Const);

#[derive(Debug, Serialize, Default)]
pub struct VerifyBitwiseXor<V: VerifyBitwiseXorSize> {
    #[serde(skip)]
    _phantom: PhantomData<V>,
}

// Asserts that the three felt expressions are in the correct range,
// and that their bitwise XOR is 0.
impl<V: VerifyBitwiseXorSize> AirFn for VerifyBitwiseXor<V> {
    type ExtIn = V;
    type In = ();
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn name(&self) -> String {
        format!("verify_bitwise_xor_{}", V::bits())
    }

    fn relation_names(&self) -> Vec<String> {
        if V::bits() == 8 {
            return get_relation_variant_names("VerifyBitwiseXor_8", 2);
        }

        vec![format!("VerifyBitwiseXor_{}", V::bits())]
    }

    fn call(
        &self,
        _air_builder: &mut AirBuilder,
        _const_input: <Self::ExtIn as ExtTable>::T,
        _: (),
    ) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            if let [a, b, c] = _const_input.to_values().expect("input has no values").as_slice() {
                assert!(
                    a.0 < (1u32 << V::bits()),
                    "RangeCheck{} failed (input {})",
                    V::bits(),
                    a.0
                );
                assert!(
                    b.0 < (1u32 << V::bits()),
                    "RangeCheck{} failed (input {})",
                    V::bits(),
                    b.0
                );
                assert!(
                    (a.0 ^ b.0) == c.0,
                    "The bitwise XOR of {:b} and {:b} is not {:b}",
                    a.0,
                    b.0,
                    c.0
                );
            } else {
                panic!(
                    "Expected 3 values, got {}",
                    _const_input.to_values().expect("input has no values").len()
                );
            }
        }
    }
}

#[macro_export]
macro_rules! new_verify_bitwise_xor {
    ($b:literal, $name:ident) => {
        #[derive(Debug, Default, Clone)]
        #[allow(non_camel_case_types)]
        pub struct $name {}

        impl VerifyBitwiseXorSize for $name {
            fn bits() -> u16 {
                $b
            }
        }

        impl ExtTable for $name {
            type T = [FeltExpr; 3];

            fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
                (0..3)
                    .map(|i| {
                        Box::new(
                            stwo_cairo_common::preprocessed_columns::bitwise_xor::BitwiseXor::new(
                                $b, i,
                            ),
                        ) as Box<dyn PreProcessedColumn>
                    })
                    .collect()
            }
        }
    };
}
