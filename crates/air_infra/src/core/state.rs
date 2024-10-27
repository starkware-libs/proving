use std::cell::RefCell;
#[cfg(test)]
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
#[cfg(test)]
use std::fmt::Display;
use std::rc::Rc;

use serde::Serialize;

use super::expressions::felt_expr::*;
#[cfg(test)]
use super::variables::*;

// Macros
#[cfg(test)]
use crate::const_expr;

#[derive(Clone, Debug, Serialize)]
pub struct State {
    row: Rc<RefCell<Vec<(FeltExpr, String)>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            row: Rc::new(RefCell::new(vec![])),
        }
    }
}

impl State {
    pub(super) fn add(&mut self, expr: &mut FeltExpr, desc: &str) {
        let len = self.row.borrow().len();
        expr.to_state(len, None);
        self.row.borrow_mut().push((expr.clone(), desc.to_string()));
    }

    pub fn get_felts(&self) -> Vec<FeltExpr> {
        self.row
            .borrow()
            .iter()
            .map(|(felt, _)| felt.clone())
            .collect()
    }
}

#[cfg(test)]
impl From<Vec<(u32, &str)>> for State {
    fn from(row: Vec<(u32, &str)>) -> Self {
        Self {
            row: Rc::new(RefCell::new(
                row.iter()
                    .map(|(x, desc)| (const_expr!(*x), desc.to_string()))
                    .collect(),
            )),
        }
    }
}

#[cfg(test)]
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        if self.row.borrow().len() != other.row.borrow().len() {
            return false;
        }
        self.row
            .borrow()
            .iter()
            .zip(other.row.borrow().iter())
            .all(|((a, sa), (b, sb))| a.calc() == b.calc() && sa == sb)
    }
}

#[cfg(test)]
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = "\n".to_string();
        for (expr, desc) in self.row.borrow().iter() {
            s.push_str(&format!("({}, \"{}\"),\n", expr.calc(), desc));
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
impl Eq for State {}
