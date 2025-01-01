use std::fmt::Debug;
use std::marker::PhantomData;

use compiled_casm_air::const_tables::{
    STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_12, STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_4,
    STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_7, STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_8,
    STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_9,
};
use inst_def::InstDef;

use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

pub trait VerifyBitwiseXorSize {
    fn bits() -> &'static u16;
}

pub fn verify_bitwise_xor(ab: &mut AirBuilder, bits: u16, input: [FeltExpr; 3]) {
    match bits {
        4 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor4>::default(), input, ()),
        7 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor7>::default(), input, ()),
        8 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor8>::default(), input, ()),
        9 => ab.lookup_call(&VerifyBitwiseXor::<VerifyBitwiseXor9>::default(), input, ()),
        12 => ab.lookup_call(
            &VerifyBitwiseXor::<VerifyBitwiseXor12>::default(),
            input,
            (),
        ),
        _ => panic!("Unsupported verify bitwise xor bits: {:?}", bits),
    }
}

#[derive(Debug, Default)]
pub struct VerifyBitwiseXor4 {}
#[derive(Debug, Default)]
pub struct VerifyBitwiseXor7 {}
#[derive(Debug, Default)]
pub struct VerifyBitwiseXor8 {}
#[derive(Debug, Default)]
pub struct VerifyBitwiseXor9 {}
#[derive(Debug, Default)]
pub struct VerifyBitwiseXor12 {}

impl VerifyBitwiseXorSize for VerifyBitwiseXor4 {
    fn bits() -> &'static u16 {
        &4
    }
}
impl VerifyBitwiseXorSize for VerifyBitwiseXor7 {
    fn bits() -> &'static u16 {
        &7
    }
}
impl VerifyBitwiseXorSize for VerifyBitwiseXor8 {
    fn bits() -> &'static u16 {
        &8
    }
}
impl VerifyBitwiseXorSize for VerifyBitwiseXor9 {
    fn bits() -> &'static u16 {
        &9
    }
}
impl VerifyBitwiseXorSize for VerifyBitwiseXor12 {
    fn bits() -> &'static u16 {
        &12
    }
}

impl ExtTable for VerifyBitwiseXor4 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_4;
    type T = [FeltExpr; 3];
}
impl ExtTable for VerifyBitwiseXor7 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_7;
    type T = [FeltExpr; 3];
}
impl ExtTable for VerifyBitwiseXor8 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_8;
    type T = [FeltExpr; 3];
}
impl ExtTable for VerifyBitwiseXor9 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_9;
    type T = [FeltExpr; 3];
}
impl ExtTable for VerifyBitwiseXor12 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_VERIFY_BITWISE_XOR_12;
    type T = [FeltExpr; 3];
}

#[derive(Debug, InstDef, Default)]
pub struct VerifyBitwiseXor<V: VerifyBitwiseXorSize + ExtTable + Debug> {
    #[instdef(skip)]
    _phantom: PhantomData<V>,
}

// Asserts that the three felt expressions are in the correct range,
// and that their bitwise XOR is 0.
impl<V: VerifyBitwiseXorSize + ExtTable + Debug> AirFn for VerifyBitwiseXor<V> {
    type ExtIn = V;
    type In = ();
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn name(&self) -> String {
        format!("verify_bitwise_xor_{}", V::bits())
    }

    fn relation_name(&self) -> Option<String> {
        Some(format!("VerifyBitwiseXor_{}", V::bits()))
    }

    fn call(
        &self,
        _air_builder: &mut AirBuilder,
        _const_input: <Self::ExtIn as ExtTable>::T,
        _: (),
    ) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            if let [a, b, c] = _const_input
                .to_values()
                .expect("input has no values")
                .as_slice()
            {
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
