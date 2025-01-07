use std::fmt::Debug;
use std::marker::PhantomData;

use inst_def::InstDef;

use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::new_range_check;

pub trait RangeCheckSize: ExtTable + Debug + Default {
    fn bits() -> &'static [u16];
}

pub fn range_check(ab: &mut AirBuilder, bits: &[u16], input: &[FeltExpr]) {
    match bits {
        [6] => call_rc::<RangeCheck6>(ab, input),
        [9] => call_rc::<RangeCheck9>(ab, input),
        [11] => call_rc::<RangeCheck11>(ab, input),
        [12] => call_rc::<RangeCheck12>(ab, input),
        [18] => call_rc::<RangeCheck18>(ab, input),
        [19] => call_rc::<RangeCheck19>(ab, input),
        [3, 6] => call_rc::<RangeCheck3_6>(ab, input),
        [4, 3] => call_rc::<RangeCheck4_3>(ab, input),
        [9, 9] => call_rc::<RangeCheck9_9>(ab, input),
        [7, 2, 5] => call_rc::<RangeCheck7_2_5>(ab, input),
        [3, 6, 6, 3] => call_rc::<RangeCheck3_6_6_3>(ab, input),
        _ => panic!("Unsupported range check bits: {:?}", bits),
    }
}

fn call_rc<R>(ab: &mut AirBuilder, input: &[FeltExpr])
where
    R: RangeCheckSize,
    <R as ExtTable>::T: TryFrom<Vec<FeltExpr>>,
    <<R as ExtTable>::T as TryFrom<Vec<FeltExpr>>>::Error: Debug,
{
    let input = input
        .to_vec()
        .try_into()
        .unwrap_or_else(|_| panic!("range check needs {} arguments", R::bits().len()));
    ab.lookup_call(&RangeCheck::<R>::default(), input, ())
}

new_range_check!([6], RangeCheck6);
new_range_check!([9], RangeCheck9);
new_range_check!([11], RangeCheck11);
new_range_check!([12], RangeCheck12);
new_range_check!([18], RangeCheck18);
new_range_check!([19], RangeCheck19);
new_range_check!([3, 6], RangeCheck3_6);
new_range_check!([4, 3], RangeCheck4_3);
new_range_check!([9, 9], RangeCheck9_9);
new_range_check!([7, 2, 5], RangeCheck7_2_5);
new_range_check!([3, 6, 6, 3], RangeCheck3_6_6_3);

#[derive(Debug, InstDef, Default)]
pub struct RangeCheck<R: RangeCheckSize> {
    #[instdef(skip)]
    pub _phantom: PhantomData<R>,
}

impl<R: RangeCheckSize> AirFn for RangeCheck<R> {
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

#[macro_export]
macro_rules! new_range_check {
    ( [$($b:literal),+], $name:ident ) => {
        #[derive(Debug, Default, Clone)]
        #[allow(non_camel_case_types)]
        pub struct $name {}

        impl RangeCheckSize for $name {
            fn bits() -> &'static [u16] {
                &[$($b),+]
            }
        }

        impl ExtTable for $name {
            const CONST_TRACE_ID: &'static str = stringify!($name);
            type T = [FeltExpr; [$($b),+].len()];
        }
    };
}
