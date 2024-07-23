use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type BoolOperation = OpExpr<Bool>;
const CHILD_NAME: &str = "as_m31";

// A variable of type Bool. Holds its name, value, and Felt representation.
#[derive(Clone, Debug)]
pub struct BoolVar {
    pub(super) name: String,
    pub(super) value: Option<Bool>,
    pub(super) as_felt: FeltExpr,
    pub(super) is_const: bool,
}

impl BoolVar {
    // Updates the Felt representation of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_as_felt(&mut self) {
        let self_as_parent = ParentExpr {
            name: self.name.clone(),
            r#type: Bool::r#type(),
            parent: None,
            index: None,
            child_name: CHILD_NAME.to_string(),
        };
        self.as_felt.set_parent(self_as_parent);
    }
}

#[derive(Clone, Debug)]
pub enum BoolExpr {
    Var(BoolVar),
    Op(BoolOperation),
}

impl BoolExpr {
    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            BoolExpr::Var(v) => &mut v.as_felt,
            BoolExpr::Op(u) => {
                if u.op == Operation::BoolFromFelt {
                    if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = &mut u.children[0] {
                        return felt_expr;
                    }
                }
                panic!("Cannot convert to a Felt");
            }
        }
    }

    pub fn as_felt(&self) -> FeltExpr {
        self.clone().as_felt_mut().clone()
    }

    // Creates a new BoolVar.
    pub fn new_var(
        name: String,
        value: Option<Bool>,
        state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = BoolVar {
            name,
            value,
            as_felt: FeltExpr::new_var(
                CHILD_NAME.to_string(),
                value.map(|v| v.as_m31()),
                state_index,
                is_const,
            ),
            is_const,
        };
        res.update_as_felt();
        res.into()
    }

    // Creates a new constant BoolVar.
    pub fn new_const(value: Bool) -> Self {
        Self::new_var(value.calc(), Some(value), None, true)
    }
}

impl Expr<Bool> for BoolExpr {
    fn value(&self) -> Option<Bool> {
        match self {
            BoolExpr::Var(v) => v.value,
            BoolExpr::Op(b) => b.value,
        }
    }
}

impl AirVar for BoolExpr {
    fn name(&self) -> String {
        match self {
            BoolExpr::Var(v) => v.name.clone(),
            BoolExpr::Op(b) => b.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt_mut()]
    }
}

impl InternalAirVarActions for BoolExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            BoolExpr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_as_felt();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for BoolExpr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            BoolExpr::Var(v) => v.as_felt.in_state(),
            BoolExpr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            BoolExpr::Var(v) => v.is_const,
            BoolExpr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

impl From<BoolVar> for BoolExpr {
    fn from(v: BoolVar) -> BoolExpr {
        BoolExpr::Var(v)
    }
}

impl From<BoolOperation> for BoolExpr {
    fn from(b: BoolOperation) -> BoolExpr {
        BoolExpr::Op(b)
    }
}

impl From<BoolExpr> for CompiledAirVar {
    fn from(expr: BoolExpr) -> CompiledAirVar {
        match expr {
            BoolExpr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return CompiledAirVar::Var(Bool::r#type(), v.name);
                }
                if v.is_const {
                    return CompiledAirVar::Const(Bool::r#type(), v.value.unwrap().calc());
                }
                CompiledAirVar::Var(Bool::r#type(), v.name)
            }
            BoolExpr::Op(op) => op.into(),
        }
    }
}

#[macro_export]
macro_rules! const_bool_expr {
    ($val:expr) => {
        BoolExpr::new_const(Bool { value: $val })
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! bool_expr {
    ($name:expr, $val:expr) => {
        BoolExpr::new_var($name.to_string(), Some(Bool::from($val)), None, false)
    };
}
