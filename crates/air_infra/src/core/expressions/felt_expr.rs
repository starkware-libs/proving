use serde::{Serialize, Serializer};

use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;

pub type FeltOperation = OpExpr<Felt>;

// A variable of type Felt. It can be a field (attribute) of another expression, like UInt16Expr, or
// a standalone variable. It can represent a felt expression that was written to the trace.
#[derive(Clone, Debug, Default)]
pub struct FeltVar {
    pub(super) name: String,
    pub(super) value: Option<Felt>,
    pub(super) state_index: Option<usize>,
    pub(super) parent: Option<(Box<ExprImpl>, Option<usize>)>,
    pub(super) is_const: bool,
}

// A felt expression can be a constant, a variable, a binary operation, or a unary operation.
#[derive(Clone, Debug)]
pub enum FeltExpr {
    Var(FeltVar),
    Op(FeltOperation),
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize) {
        assert!(!self.name().starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
        assert!(!self.is_const());

        let name = format!("state[{}]", index);
        let value = self.value();
        match self {
            FeltExpr::Var(v) => {
                v.name = name;
                v.state_index = Some(index)
            }
            _ => *self = Self::new_var(name, value, Some(index), false),
        }
    }

    pub fn set_parent(&mut self, parent: ExprImpl, index: Option<usize>) {
        if let FeltExpr::Var(v) = self {
            v.parent = Some((Box::new(parent), index));
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new FeltVar.
    pub fn new_var(
        name: String,
        value: Option<Felt>,
        state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        FeltVar {
            name,
            value,
            state_index,
            parent: None,
            is_const,
        }
        .into()
    }

    // Creates a new constant FeltVar.
    pub fn new_const(value: Felt) -> Self {
        Self::new_var(value.calc(), Some(value), None, true)
    }

    pub fn let_for_constraint(&self, name: String) -> Self {
        assert!(name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));

        Self::new_var(name, self.value(), None, self.is_const())
    }
}

impl Expr<Felt> for FeltExpr {
    fn value(&self) -> Option<Felt> {
        match self {
            FeltExpr::Var(v) => v.value,
            FeltExpr::Op(b) => b.value,
        }
    }
}

impl AirVar for FeltExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn name(&self) -> String {
        match self {
            FeltExpr::Var(v) => v.name.clone(),
            FeltExpr::Op(b) => b.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            FeltExpr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            FeltExpr::Var(v) => {
                v.state_index.is_some()
                    || v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.is_const
            }
            FeltExpr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self]
    }

    fn is_const(&self) -> bool {
        match self {
            FeltExpr::Var(v) => v.is_const,
            FeltExpr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

impl Default for FeltExpr {
    fn default() -> Self {
        FeltExpr::Var(FeltVar::default())
    }
}

impl From<FeltVar> for FeltExpr {
    fn from(variable: FeltVar) -> FeltExpr {
        FeltExpr::Var(variable)
    }
}

impl From<FeltOperation> for FeltExpr {
    fn from(binary: FeltOperation) -> FeltExpr {
        FeltExpr::Op(binary)
    }
}

impl From<FeltExpr> for GenericAirVar {
    fn from(expr: FeltExpr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<FeltExpr> for ProcessedAirVar {
    fn from(expr: FeltExpr) -> ProcessedAirVar {
        match expr {
            FeltExpr::Var(v) => {
                // v is an intermediate variable
                if v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX)
                {
                    return ProcessedAirVar::Var(Felt::r#type(), v.name);
                }

                // v is a constant
                if v.is_const {
                    return ProcessedAirVar::Const(Felt::r#type(), v.value.unwrap().calc());
                }

                // v was written to the trace
                if let Some(i) = v.state_index {
                    return ProcessedAirVar::State(i);
                }

                // v is a field of another variable
                if let Some((var, index)) = v.parent {
                    if let Some(i) = index {
                        let index_var = ProcessedAirVar::Const("usize".to_string(), i.to_string());
                        return ProcessedAirVar::MethodCall(
                            Box::new((*var).into()),
                            v.name,
                            vec![index_var],
                        );
                    }
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }

                // v is a standalone variable
                ProcessedAirVar::Var(Felt::r#type(), v.name)
            }
            FeltExpr::Op(op) => op.into(),
        }
    }
}

impl Serialize for FeltExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let var: ProcessedAirVar = self.clone().into();
        serializer.collect_str(&var.to_string())
    }
}

#[macro_export]
macro_rules! const_expr {
    ($val:expr) => {
        FeltExpr::new_const(Felt::from_u32_unchecked($val))
    };
}

#[macro_export]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None, false)
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), Some(0), false)
        } else {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None, false)
        }
    };
}
