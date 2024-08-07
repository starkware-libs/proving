use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type UInt16Operation = OpExpr<UInt16>;
const CHILD_NAME: &str = "as_m31";

// A variable of type UInt16. Holds its name, value, and Felt representation.
// It can be a field (attribute) of another expression, like UInt32Expr, or
// a standalone variable.
#[derive(Clone, Debug)]
pub struct UInt16Var {
    pub(super) name: String,
    pub(super) value: Option<UInt16>,
    pub(super) as_felt: FeltExpr,
    pub(super) parent: Option<ParentExpr>,
    pub(super) is_const: bool,
}

impl UInt16Var {
    // Updates the Felt representation of the variable.
    // Called whenever a variable is created (see new_var, let_for_deduction and set_parent).
    fn update_as_felt(&mut self) {
        let self_as_parent = ParentExpr {
            name: self.name.clone(),
            r#type: UInt16::r#type(),
            parent: self.parent.clone().map(Box::new),
            index: None,
            child_name: CHILD_NAME.to_string(),
        };
        self.as_felt.set_parent(self_as_parent);
    }
}

#[derive(Clone, Debug)]
pub enum UInt16Expr {
    Var(UInt16Var),
    Op(UInt16Operation),
}

impl UInt16Expr {
    pub fn as_felt_mut(&mut self) -> &mut FeltExpr {
        match self {
            UInt16Expr::Var(v) => &mut v.as_felt,
            UInt16Expr::Op(op) => {
                if op.op == Operation::UInt16FromBool {
                    if let AirVarImpl::Expr(ExprImpl::Bool(bool_expr)) = &mut op.children[0] {
                        return bool_expr.as_felt_mut();
                    }
                } else if op.op == Operation::UInt16FromFelt {
                    if let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = &mut op.children[0] {
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

    // Called whenever a parent variable is created (see update_parts of UInt32Expr).
    pub(super) fn set_parent(&mut self, parent: ParentExpr) {
        if let UInt16Expr::Var(v) = self {
            v.parent = Some(parent);
            v.update_as_felt();
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new UInt16Var.
    pub fn new_var(
        name: String,
        value: Option<UInt16>,
        state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = UInt16Var {
            name,
            value,
            as_felt: FeltExpr::new_var(
                CHILD_NAME.to_string(),
                value.map(|v| v.as_m31()),
                state_index,
                is_const,
            ),
            parent: None,
            is_const,
        };
        res.update_as_felt();
        res.into()
    }

    // Creates a new constant UInt16Var.
    pub fn new_const(value: UInt16) -> Self {
        Self::new_var(value.calc(), Some(value), None, true)
    }
}

impl Expr<UInt16> for UInt16Expr {
    fn value(&self) -> Option<UInt16> {
        match self {
            UInt16Expr::Var(v) => v.value,
            UInt16Expr::Op(op) => op.value,
        }
    }
}

impl AirVar for UInt16Expr {
    fn name(&self) -> String {
        match self {
            UInt16Expr::Var(v) => v.name.clone(),
            UInt16Expr::Op(op) => op.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt_mut()]
    }
}

impl InternalAirVarActions for UInt16Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt16Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_as_felt();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for UInt16Expr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            UInt16Expr::Var(v) => v.as_felt.in_state(),
            UInt16Expr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            UInt16Expr::Var(v) => v.is_const,
            UInt16Expr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

impl From<UInt16Var> for UInt16Expr {
    fn from(v: UInt16Var) -> UInt16Expr {
        UInt16Expr::Var(v)
    }
}

impl From<UInt16Operation> for UInt16Expr {
    fn from(b: UInt16Operation) -> UInt16Expr {
        UInt16Expr::Op(b)
    }
}

impl From<UInt16Expr> for CompiledAirVar {
    fn from(expr: UInt16Expr) -> CompiledAirVar {
        match expr {
            UInt16Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return CompiledAirVar::Var(UInt16::r#type(), v.name);
                }
                if v.is_const {
                    return CompiledAirVar::Const(UInt16::r#type(), v.value.unwrap().calc());
                }
                if let Some(parent) = v.parent {
                    return parent.get_compiled_child();
                }

                CompiledAirVar::Var(UInt16::r#type(), v.name)
            }
            UInt16Expr::Op(op) => op.into(),
        }
    }
}

#[macro_export]
macro_rules! const_u16_expr {
    ($val:expr) => {
        UInt16Expr::new_const(UInt16 { value: $val })
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! u16_expr {
    ($name:expr, $val:expr) => {
        UInt16Expr::new_var($name.to_string(), Some(UInt16::from($val)), None, false)
    };
}
