use std::fmt::Debug;
use std::marker::PhantomData;

use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;

use super::seq::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::new_range_check;

const STWO_COMPONENT_TYPE_RANGE_CHECK: &str = "RangeCheck";

pub trait RangeCheckSize: ExtTable + Debug + Default {
    fn bits() -> &'static [u16];
}

#[derive(Debug, Copy, Clone, Serialize, Default, PartialEq, Eq)]
pub enum RCVariant {
    #[default]
    A,
    B,
    C,
    D,
}

pub fn range_check(ab: &mut AirBuilder, bits: &[u16], input: &[FeltExpr]) {
    match bits {
        [6] => call_rc::<SeqConstLen<6>>(ab, input, RCVariant::default()),
        [8] => call_rc::<SeqConstLen<8>>(ab, input, RCVariant::default()),
        [9] => call_rc::<SeqConstLen<9>>(ab, input, RCVariant::default()),
        [11] => call_rc::<SeqConstLen<11>>(ab, input, RCVariant::default()),
        [12] => call_rc::<SeqConstLen<12>>(ab, input, RCVariant::default()),
        [18] => call_rc::<SeqConstLen<18>>(ab, input, RCVariant::default()),
        [19] => call_rc::<SeqConstLen<19>>(ab, input, RCVariant::default()),
        [4, 3] => call_rc::<RangeCheck_4_3_Const>(ab, input, RCVariant::default()),
        [5, 4] => call_rc::<RangeCheck_5_4_Const>(ab, input, RCVariant::default()),
        [9, 9] => call_rc::<RangeCheck_9_9_Const>(ab, input, RCVariant::default()),
        [7, 2, 5] => call_rc::<RangeCheck_7_2_5_Const>(ab, input, RCVariant::default()),
        [3, 6, 6, 3] => call_rc::<RangeCheck_3_6_6_3_Const>(ab, input, RCVariant::default()),
        [4, 4, 4, 4] => call_rc::<RangeCheck_4_4_4_4_Const>(ab, input, RCVariant::default()),
        [4, 4] => call_rc::<RangeCheck_4_4_Const>(ab, input, RCVariant::default()),
        [3, 3, 3, 3, 3] => call_rc::<RangeCheck_3_3_3_3_3_Const>(ab, input, RCVariant::default()),
        [2, 2, 2, 2, 2] => call_rc::<RangeCheck_2_2_2_2_2_Const>(ab, input, RCVariant::default()),
        _ => panic!("Unsupported range check bits: {:?}", bits),
    }
}

pub fn range_check_variant(ab: &mut AirBuilder, bits: &[u16], input: &[FeltExpr], variant: usize) {
    let variant = match variant {
        0 => RCVariant::A,
        1 => RCVariant::B,
        2 => RCVariant::C,
        3 => RCVariant::D,
        _ => unreachable!(),
    };

    match bits {
        [19] => call_rc::<SeqConstLen<19>>(ab, input, variant),
        [9, 9] => call_rc::<RangeCheck_9_9_Const>(ab, input, variant),
        _ => panic!("Unsupported range check bits: {:?}", bits),
    }
}

fn call_rc<R>(ab: &mut AirBuilder, input: &[FeltExpr], variant: RCVariant)
where
    R: RangeCheckSize,
    <R as ExtTable>::T: TryFrom<Vec<FeltExpr>>,
    <<R as ExtTable>::T as TryFrom<Vec<FeltExpr>>>::Error: Debug,
{
    let input = input
        .to_vec()
        .try_into()
        .unwrap_or_else(|_| panic!("range check needs {} arguments", R::bits().len()));
    ab.lookup_call(
        &RangeCheck::<R> {
            _phantom: PhantomData,
            variant,
        },
        input,
        (),
    )
}

new_range_check!([4, 3], RangeCheck_4_3_Const);
new_range_check!([5, 4], RangeCheck_5_4_Const);
new_range_check!([9, 9], RangeCheck_9_9_Const);
new_range_check!([7, 2, 5], RangeCheck_7_2_5_Const);
new_range_check!([3, 6, 6, 3], RangeCheck_3_6_6_3_Const);
new_range_check!([4, 4, 4, 4], RangeCheck_4_4_4_4_Const);
new_range_check!([4, 4], RangeCheck_4_4_Const);
new_range_check!([3, 3, 3, 3, 3], RangeCheck_3_3_3_3_3_Const);
new_range_check!([2, 2, 2, 2, 2], RangeCheck_2_2_2_2_2_Const);

#[derive(Debug, Serialize, Default)]
pub struct RangeCheck<R: RangeCheckSize> {
    #[serde(skip)]
    pub _phantom: PhantomData<R>,
    pub variant: RCVariant,
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
        match self.variant {
            RCVariant::A => format!("range_check_{}", bits),
            RCVariant::B => format!("range_check_{}_b", bits),
            RCVariant::C => format!("range_check_{}_c", bits),
            RCVariant::D => format!("range_check_{}_d", bits),
        }
    }

    fn relation_name(&self) -> Option<String> {
        let bits = R::bits()
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("_");
        match self.variant {
            RCVariant::A => Some(format!("RangeCheck_{}", bits)),
            RCVariant::B => Some(format!("RangeCheck_{}_B", bits)),
            RCVariant::C => Some(format!("RangeCheck_{}_C", bits)),
            RCVariant::D => Some(format!("RangeCheck_{}_D", bits)),
        }
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
    ( [$b0:literal,$($b:literal),+], $name:ident ) => {
        #[derive(Debug, Default, Clone)]
        #[allow(non_camel_case_types)]
        pub struct $name {}

        impl RangeCheckSize for $name {
            fn bits() -> &'static [u16] {
                &[$b0,$($b),+]
            }
        }

        impl ExtTable for $name {
            const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_RANGE_CHECK;
            type T = [FeltExpr; [$b0,$($b),+].len()];

            fn args() -> Vec<String> {
                vec![format!("[{}]", [$b0.to_string(), $($b.to_string()),+].join(","))]
            }

            fn generic_param() -> Option<u32> {
                Some([$b0,$($b),+].len() as u32)
            }
        }
    };
}
