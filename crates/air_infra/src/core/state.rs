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
pub struct StateCell(FeltExpr, Option<String>);

#[derive(Clone, Debug, Serialize)]
pub struct State {
    row: Rc<RefCell<Vec<StateCell>>>,
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
        let desc = (!desc.is_empty()).then(|| desc.to_string());
        expr.to_state(StateInfo::StateIndex(len, desc.clone()));
        self.row.borrow_mut().push(StateCell(expr.clone(), desc));
    }

    pub fn get_felts(&self) -> Vec<FeltExpr> {
        self.row
            .borrow()
            .iter()
            .map(|cell| cell.0.clone())
            .collect()
    }

    pub(super) fn get_cell_name(index: usize, desc: &Option<String>) -> String {
        match desc {
            Some(desc) => format!("{}_col{}", desc, index),
            None => format!("col{}", index),
        }
    }

    pub(super) fn get_state_names(&self) -> Vec<String> {
        self.row
            .borrow()
            .iter()
            .enumerate()
            .map(|(i, cell)| Self::get_cell_name(i, &cell.1))
            .collect()
    }
}

#[cfg(test)]
impl From<Vec<(u32, &str)>> for State {
    fn from(row: Vec<(u32, &str)>) -> Self {
        Self {
            row: Rc::new(RefCell::new(
                row.iter()
                    .map(|(x, desc)| {
                        StateCell(
                            const_expr!(*x),
                            (!desc.is_empty()).then(|| desc.to_string()),
                        )
                    })
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
            .all(|(a, b)| a.0.calc() == b.0.calc() && a.1 == b.1)
    }
}

#[cfg(test)]
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = "\n".to_string();
        for cell in self.row.borrow().iter() {
            s.push_str(&format!(
                "({}, \"{}\"),\n",
                cell.0.calc(),
                cell.1.clone().unwrap_or_default()
            ));
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
impl Eq for State {}
