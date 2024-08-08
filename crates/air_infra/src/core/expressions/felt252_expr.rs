use std::array::from_fn;

use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

// Macros
use crate::const_felt252_expr;

pub type Felt252Operation = OpExpr<Felt252>;
const CHILD_NAME: &str = "get_m31";

// A variable of type Felt252. Holds its name, and value. It is represented as FELT252_N_WORDS felts,
// FELT252_BITS_PER_WORD bits each.
#[derive(Clone, Debug)]
pub struct Felt252Var {
    pub(super) name: String,
    pub(super) value: Option<Felt252>,
    pub(super) felts: [FeltExpr; FELT252_N_WORDS],
    pub(super) is_const: bool,
}

impl Felt252Var {
    // Updates the Felts representation of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        for (index, felt) in self.felts.iter_mut().enumerate() {
            let self_as_parent = ParentExpr {
                name: self.name.clone(),
                r#type: Felt252::r#type(),
                parent: None,
                index: Some(index),
                child_name: CHILD_NAME.to_string(),
            };
            felt.set_parent(self_as_parent);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Felt252Expr {
    Var(Felt252Var),
    Op(Felt252Operation),
}

impl Felt252Expr {
    // Creates a new Felt252Var.
    pub fn new_var(
        name: String,
        value: Option<Felt252>,
        state_indices: Option<[usize; FELT252_N_WORDS]>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = Felt252Var {
            name,
            value,
            felts: from_fn(|i| {
                FeltExpr::new_var(
                    CHILD_NAME.to_string(),
                    value.map(|v| v.get_m31(i)),
                    state_indices.map(|is| is[i]),
                    is_const,
                )
            }),
            is_const,
        };
        res.update_parts();
        res.into()
    }

    // Creates a new constant Felt252Var.
    pub fn new_const(value: Felt252) -> Self {
        Self::new_var(value.calc(), Some(value), None, true)
    }

    pub fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        match self {
            Felt252Expr::Var(v) => v.felts.get_mut(index).expect("index out of bounds"),
            Felt252Expr::Op(op) => {
                if op.op == Operation::Felt252FromFeltsArray {
                    if let AirVarImpl::Array(arr) = &mut op.children[0] {
                        if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) =
                            arr.get_mut(index).expect("index out of bounds")
                        {
                            return felt_expr;
                        }
                    }
                }
                panic!("Cannot convert to felts");
            }
        }
    }

    pub fn get_felt(&self, index: usize) -> FeltExpr {
        self.clone().get_felt_mut(index).clone()
    }
}

impl Expr<Felt252> for Felt252Expr {
    fn value(&self) -> Option<Felt252> {
        match self {
            Felt252Expr::Var(v) => v.value,
            Felt252Expr::Op(op) => op.value,
        }
    }
}

impl AirVar for Felt252Expr {
    fn name(&self) -> String {
        match self {
            Felt252Expr::Var(v) => v.name.clone(),
            Felt252Expr::Op(op) => op.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Expr::Var(v) => {
                let mut res = Vec::new();
                for felt in v.felts.iter_mut() {
                    res.push(felt);
                }
                res
            }
            Felt252Expr::Op(op) => {
                if op.op == Operation::Felt252FromFeltsArray {
                    if let AirVarImpl::Array(arr) = &mut op.children[0] {
                        let len = arr.len();
                        let mut felts = vec![];
                        for g in arr {
                            if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = g {
                                felts.push(felt_expr);
                            }
                        }
                        if felts.len() == len {
                            return felts;
                        }
                    }
                }
                panic!("Cannot convert to felts");
            }
        }
    }
}

impl InternalAirVarActions for Felt252Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn let_(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            Felt252Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for Felt252Expr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            Felt252Expr::Var(v) => v.felts.iter().all(|f| f.in_state()),
            Felt252Expr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            Felt252Expr::Var(v) => v.is_const,
            Felt252Expr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

// Default is implemented for Felt252Expr because it is stored in memory.
impl Default for Felt252Expr {
    fn default() -> Self {
        const_felt252_expr!(0, 0)
    }
}

impl From<Felt252Var> for Felt252Expr {
    fn from(v: Felt252Var) -> Felt252Expr {
        Felt252Expr::Var(v)
    }
}

impl From<Felt252Operation> for Felt252Expr {
    fn from(b: Felt252Operation) -> Felt252Expr {
        Felt252Expr::Op(b)
    }
}

impl From<Felt252Expr> for CompiledAirVar {
    fn from(expr: Felt252Expr) -> CompiledAirVar {
        match expr {
            Felt252Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return CompiledAirVar::Var(Felt252::r#type(), v.name);
                }
                if v.is_const {
                    return CompiledAirVar::Const(Felt252::r#type(), v.value.unwrap().calc());
                }
                CompiledAirVar::Var(Felt252::r#type(), v.name)
            }
            Felt252Expr::Op(op) => op.into(),
        }
    }
}

#[macro_export]
macro_rules! const_felt252_expr {
    ($low:expr, $high:expr) => {
        Felt252Expr::new_const(($low, $high).into())
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! felt252_expr {
    ($name:expr, $low:expr, $high:expr) => {
        Felt252Expr::new_var(
            $name.to_string(),
            Some(Felt252::from(($low, $high))),
            None,
            false,
        )
    };
}
