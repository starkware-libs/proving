use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::M31;

use super::expressions::felt_expr::*;
use crate::core::expressions::var_expr::*;

#[derive(Debug, Clone, Default)]
pub struct PublicParams {
    values: Rc<RefCell<HashMap<PublicParam, FeltExpr>>>,
}

impl PublicParams {
    fn create_public_param_expr(param: PublicParam, value: Option<M31>) -> FeltExpr {
        let mut result: FeltExpr = VarExpr::new("".to_string(), value, false, None).into();
        result.set_value(ValueInfo::PublicParam(param));
        result
    }

    #[cfg(any(test, feature = "test"))]
    pub fn set(&mut self, param: PublicParam, value: M31) {
        self.values
            .borrow_mut()
            .insert(param.clone(), Self::create_public_param_expr(param, Some(value)));
    }

    pub fn get(&self, param: PublicParam) -> FeltExpr {
        // If we don't have a value for this parameter, return a newly-created, value-less
        // parameter. This should only happen in build mode.
        return self
            .values
            .borrow()
            .get(&param)
            .unwrap_or(&Self::create_public_param_expr(param, None))
            .clone();
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize, PartialOrd, Ord)]
pub enum PublicParam {
    AddModBuiltinSegmentStart,
    MulModBuiltinSegmentStart,
    BitwiseBuiltinSegmentStart,
    PoseidonBuiltinSegmentStart,
    RangeCheckBuiltinSegmentStart,
    RangeCheck96BuiltinSegmentStart,
    PedersenBuiltinSegmentStart,
    ECOpBuiltinSegmentStart,
}

impl PublicParam {
    pub fn name(&self) -> String {
        match self {
            PublicParam::AddModBuiltinSegmentStart => "add_mod_builtin_segment_start".to_string(),
            PublicParam::MulModBuiltinSegmentStart => "mul_mod_builtin_segment_start".to_string(),
            PublicParam::BitwiseBuiltinSegmentStart => "bitwise_builtin_segment_start".to_string(),
            PublicParam::PoseidonBuiltinSegmentStart => {
                "poseidon_builtin_segment_start".to_string()
            }
            PublicParam::RangeCheckBuiltinSegmentStart => {
                "range_check_builtin_segment_start".to_string()
            }
            PublicParam::RangeCheck96BuiltinSegmentStart => {
                "range_check96_builtin_segment_start".to_string()
            }
            PublicParam::PedersenBuiltinSegmentStart => {
                "pedersen_builtin_segment_start".to_string()
            }
            PublicParam::ECOpBuiltinSegmentStart => "ec_op_builtin_segment_start".to_string(),
        }
    }
}
