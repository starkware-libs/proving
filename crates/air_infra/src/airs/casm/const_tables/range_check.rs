use indexmap::IndexMap;

use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
#[cfg(test)]
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_RANGE_CHECK_2: &str = "RangeCheck2";
const STWO_COMPONENT_TYPE_RANGE_CHECK_3: &str = "RangeCheck3";
const STWO_COMPONENT_TYPE_RANGE_CHECK_4: &str = "RangeCheck4";
const STWO_COMPONENT_TYPE_RANGE_CHECK_5: &str = "RangeCheck5";
const STWO_COMPONENT_TYPE_RANGE_CHECK_6: &str = "RangeCheck6";
const STWO_COMPONENT_TYPE_RANGE_CHECK_7: &str = "RangeCheck7";
const STWO_COMPONENT_TYPE_RANGE_CHECK_8: &str = "RangeCheck8";
const STWO_COMPONENT_TYPE_RANGE_CHECK_9: &str = "RangeCheck9";
const STWO_COMPONENT_TYPE_RANGE_CHECK_16: &str = "RangeCheck16";
const STWO_COMPONENT_TYPE_RANGE_CHECK_VECTOR_2_5: &str = "RangeCheckVector_2_5";
const STWO_COMPONENT_TYPE_RANGE_CHECK_VECTOR_4_3: &str = "RangeCheckVector_4_3";

#[derive(Debug)]
pub struct RangeCheck<const N: usize> {
    pub bits: [u16; N],
}

impl<const N: usize> AirFn for RangeCheck<N> {
    type In = [FeltExpr; N];
    type Out = ();

    fn const_input(&self) -> Option<String> {
        match self.bits.as_slice() {
            // Note: Each specific rc in the list must be implemented in stwo by a component of
            // the same name.
            [2] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_2.to_string()),
            [3] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_3.to_string()),
            [4] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_4.to_string()),
            [5] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_5.to_string()),
            [6] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_6.to_string()),
            [7] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_7.to_string()),
            [8] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_8.to_string()),
            [9] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_9.to_string()),
            [16] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_16.to_string()),
            [2, 5] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_VECTOR_2_5.to_string()),
            [4, 3] => Some(STWO_COMPONENT_TYPE_RANGE_CHECK_VECTOR_4_3.to_string()),
            _ => panic!("Invalid range check bits {:?}.", self.bits),
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [("bits".to_string(), format!("{:?}", self.bits))].into()
    }

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            for (index, (&input, &bits)) in _input
                .to_values()
                .expect("input has no values")
                .iter()
                .zip(self.bits.iter())
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
