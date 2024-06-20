use std::collections::BTreeMap;

use crate::core::air_fn::{AirFn, TraceType};
use crate::core::expressions::felt_expr::FeltExpr;

const STWO_COMPONENT_TYPE_RANGE_CHECK_3: &str = "RangeCheck3";
const STWO_COMPONENT_TYPE_RANGE_CHECK_4: &str = "RangeCheck4";
const STWO_COMPONENT_TYPE_RANGE_CHECK_8: &str = "RangeCheck8";
const STWO_COMPONENT_TYPE_RANGE_CHECK_12: &str = "RangeCheck12";
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
            3 => STWO_COMPONENT_TYPE_RANGE_CHECK_3.to_string(),
            4 => STWO_COMPONENT_TYPE_RANGE_CHECK_4.to_string(),
            8 => STWO_COMPONENT_TYPE_RANGE_CHECK_8.to_string(),
            12 => STWO_COMPONENT_TYPE_RANGE_CHECK_12.to_string(),
            16 => STWO_COMPONENT_TYPE_RANGE_CHECK_16.to_string(),
            _ => panic!("Invalid range check bits!"),
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [("bits".to_string(), self.bits.to_string())].into()
    }

    fn call(
        &self,
        _air_builder: &mut crate::core::air_fn::AirBuilder,
        _input: Self::In,
    ) -> Self::Out {
        // TODO: In run mode, assert input < 2**self.bits
        Self::Out::default()
    }
}
