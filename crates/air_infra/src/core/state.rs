use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use serde::Serialize;

use super::expressions::felt_expr::*;
#[cfg(test)]
use super::variables::*;

#[derive(Clone, Debug, Serialize)]
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
    pub(super) fn add(&self, expr: &mut FeltExpr) {
        let len = self.row.borrow().len();
        expr.to_state(len, None);
        self.row.borrow_mut().push(expr.clone());
    }

    #[cfg(test)]
    pub fn calc(&self) -> Vec<String> {
        self.row.borrow().iter().map(|x| x.calc()).collect()
    }
}
