use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use compiled_casm_air::public_params::PublicParam;
use stwo_cairo_common::prover_types::cpu::M31;

use super::expressions::felt_expr::*;
use super::variables::*;
use crate::core::expressions::var_expr::*;

#[derive(Debug, Clone, Default)]
pub struct PublicParams {
    values: Rc<RefCell<HashMap<PublicParam, FeltExpr>>>,
}

impl PublicParams {
    fn create_public_param_expr(param: PublicParam, value: Option<M31>) -> FeltExpr {
        let mut result: FeltExpr =
            VarExpr::new("".to_string(), value, false, false, Visibility::default()).into();
        result.to_state(StateInfo::PublicParam(param));
        result
    }

    #[cfg(test)]
    pub fn set(&mut self, param: PublicParam, value: M31) {
        self.values.borrow_mut().insert(
            param.clone(),
            Self::create_public_param_expr(param, Some(value)),
        );
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
