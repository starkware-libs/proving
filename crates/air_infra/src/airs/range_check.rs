use std::collections::BTreeMap;

use crate::core::air_fn::{AirFn, TraceType};

#[allow(unused_imports)] // import only used in cfg(test)
use crate::core::expressions::expr::Expr;
use crate::core::expressions::felt_expr::FeltExpr;

#[allow(unused_imports)] // import only used in cfg(test)
use crate::core::variables::*;

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
