use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::expressions::expr::Expr;
use super::expressions::felt_expr::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub row: Rc<RefCell<Vec<FeltExpr>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            row: Rc::new(RefCell::new(vec![])),
        }
    }
}

impl State {
    #[allow(unused)]
    pub(super) fn add(&self, expr: &mut FeltExpr) {
        let len = self.row.borrow().len();
        expr.to_state(len);
        self.row.borrow_mut().push(expr.clone());
    }

    #[cfg(test)]
    pub fn calc(&self) -> Vec<String> {
        self.row.borrow().iter().map(|x| x.calc()).collect()
    }
}
