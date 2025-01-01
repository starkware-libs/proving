use std::fmt::Debug;
use std::marker::PhantomData;

use compiled_casm_air::const_tables::{
    STWO_COMPONENT_TYPE_RANGE_CHECK_11, STWO_COMPONENT_TYPE_RANGE_CHECK_19,
    STWO_COMPONENT_TYPE_RANGE_CHECK_3_6, STWO_COMPONENT_TYPE_RANGE_CHECK_3_6_6_3,
    STWO_COMPONENT_TYPE_RANGE_CHECK_4_3, STWO_COMPONENT_TYPE_RANGE_CHECK_6,
    STWO_COMPONENT_TYPE_RANGE_CHECK_7_2_5, STWO_COMPONENT_TYPE_RANGE_CHECK_9,
    STWO_COMPONENT_TYPE_RANGE_CHECK_9_9,
};
use inst_def::InstDef;

use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

pub trait RangeCheckSize {
    fn bits() -> &'static [u16];
}

pub fn range_check(ab: &mut AirBuilder, bits: &[u16], input: &[FeltExpr]) {
    match bits {
        [6] => ab.lookup_call(
            &RangeCheck::<RangeCheck6>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 1 argument"),
            (),
        ),
        [9] => ab.lookup_call(
            &RangeCheck::<RangeCheck9>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 1 argument"),
            (),
        ),
        [11] => ab.lookup_call(
            &RangeCheck::<RangeCheck11>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 1 argument"),
            (),
        ),
        [19] => ab.lookup_call(
            &RangeCheck::<RangeCheck19>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 1 argument"),
            (),
        ),
        [3, 6] => ab.lookup_call(
            &RangeCheck::<RangeCheck3_6>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 2 argument"),
            (),
        ),
        [4, 3] => ab.lookup_call(
            &RangeCheck::<RangeCheck4_3>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 2 argument"),
            (),
        ),
        [9, 9] => ab.lookup_call(
            &RangeCheck::<RangeCheck9_9>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 2 argument"),
            (),
        ),
        [7, 2, 5] => ab.lookup_call(
            &RangeCheck::<RangeCheck7_2_5>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 3 argument"),
            (),
        ),
        [3, 6, 6, 3] => ab.lookup_call(
            &RangeCheck::<RangeCheck3_6_6_3>::default(),
            input
                .to_vec()
                .try_into()
                .expect("Range check needs 4 argument"),
            (),
        ),
        _ => panic!("Unsupported range check bits: {:?}", bits),
    }
}

#[derive(Debug, Default)]
pub struct RangeCheck6 {}
#[derive(Debug, Default)]
pub struct RangeCheck9 {}
#[derive(Debug, Default)]
pub struct RangeCheck11 {}
#[derive(Debug, Default)]
pub struct RangeCheck19 {}
#[derive(Debug, Default)]
pub struct RangeCheck3_6 {}
#[derive(Debug, Default)]
pub struct RangeCheck3_6_6_3 {}
#[derive(Debug, Default)]
pub struct RangeCheck4_3 {}
#[derive(Debug, Default)]
pub struct RangeCheck9_9 {}
#[derive(Debug, Default)]
pub struct RangeCheck7_2_5 {}

impl RangeCheckSize for RangeCheck6 {
    fn bits() -> &'static [u16] {
        &[6]
    }
}
impl RangeCheckSize for RangeCheck9 {
    fn bits() -> &'static [u16] {
        &[9]
    }
}
impl RangeCheckSize for RangeCheck11 {
    fn bits() -> &'static [u16] {
        &[11]
    }
}
impl RangeCheckSize for RangeCheck19 {
    fn bits() -> &'static [u16] {
        &[19]
    }
}
impl RangeCheckSize for RangeCheck3_6 {
    fn bits() -> &'static [u16] {
        &[3, 6]
    }
}
impl RangeCheckSize for RangeCheck3_6_6_3 {
    fn bits() -> &'static [u16] {
        &[3, 6, 6, 3]
    }
}
impl RangeCheckSize for RangeCheck4_3 {
    fn bits() -> &'static [u16] {
        &[4, 3]
    }
}
impl RangeCheckSize for RangeCheck9_9 {
    fn bits() -> &'static [u16] {
        &[9, 9]
    }
}
impl RangeCheckSize for RangeCheck7_2_5 {
    fn bits() -> &'static [u16] {
        &[7, 2, 5]
    }
}

impl ExtTable for RangeCheck6 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_6;
    type T = [FeltExpr; 1];
}
impl ExtTable for RangeCheck9 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_9;
    type T = [FeltExpr; 1];
}
impl ExtTable for RangeCheck11 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_11;
    type T = [FeltExpr; 1];
}
impl ExtTable for RangeCheck19 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_19;
    type T = [FeltExpr; 1];
}
impl ExtTable for RangeCheck3_6 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_3_6;
    type T = [FeltExpr; 2];
}
impl ExtTable for RangeCheck3_6_6_3 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_3_6_6_3;
    type T = [FeltExpr; 4];
}
impl ExtTable for RangeCheck4_3 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_4_3;
    type T = [FeltExpr; 2];
}
impl ExtTable for RangeCheck9_9 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_9_9;
    type T = [FeltExpr; 2];
}
impl ExtTable for RangeCheck7_2_5 {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK_7_2_5;
    type T = [FeltExpr; 3];
}

#[derive(Debug, InstDef, Default)]
pub struct RangeCheck<R: RangeCheckSize + ExtTable + Debug> {
    #[instdef(skip)]
    pub _phantom: PhantomData<R>,
}

impl<R: RangeCheckSize + ExtTable + Debug> AirFn for RangeCheck<R> {
    type ExtIn = R;
    type In = ();
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn name(&self) -> String {
        let bits = R::bits()
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("_");
        format!("range_check_{}", bits)
    }

    fn relation_name(&self) -> Option<String> {
        let bits = R::bits()
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("_");
        Some(format!("RangeCheck_{}", bits))
    }

    fn call(
        &self,
        _air_builder: &mut AirBuilder,
        _const_input: <Self::ExtIn as ExtTable>::T,
        _: (),
    ) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            for (index, (&input, &bits)) in _const_input
                .to_values()
                .expect("input has no values")
                .iter()
                .zip(R::bits().iter())
                .enumerate()
            {
                assert!(
                    input.0 < (1u32 << bits),
                    "RangeCheck failed on element {}: RangeCheck{} on input {}",
                    index,
                    bits,
                    input.0
                );
            }
        }
    }
}
