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

#[derive(Debug)]
pub struct RangeCheck {
    pub bits: u16,
}

impl AirFn for RangeCheck {
    type In = FeltExpr;
    type Out = ();

    fn name(&self) -> String {
        match self.bits {
            // Note: Each specific rc in the list must be implemented in stwo by a component of
            // the same name.
            2 => STWO_COMPONENT_TYPE_RANGE_CHECK_2.to_string(),
            3 => STWO_COMPONENT_TYPE_RANGE_CHECK_3.to_string(),
            4 => STWO_COMPONENT_TYPE_RANGE_CHECK_4.to_string(),
            5 => STWO_COMPONENT_TYPE_RANGE_CHECK_5.to_string(),
            6 => STWO_COMPONENT_TYPE_RANGE_CHECK_6.to_string(),
            7 => STWO_COMPONENT_TYPE_RANGE_CHECK_7.to_string(),
            8 => STWO_COMPONENT_TYPE_RANGE_CHECK_8.to_string(),
            9 => STWO_COMPONENT_TYPE_RANGE_CHECK_9.to_string(),
            16 => STWO_COMPONENT_TYPE_RANGE_CHECK_16.to_string(),
            _ => panic!("Invalid range check bits {}.", self.bits),
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [("bits".to_string(), self.bits.to_string())].into()
    }

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let in_value = _input.to_values()[0].0;
            assert!(
                in_value < (1u32 << self.bits),
                "RangeCheck{} failed (input {})",
                self.bits,
                in_value
            );
        }
    }
}
