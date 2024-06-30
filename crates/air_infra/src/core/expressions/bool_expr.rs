use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type BoolBinary = BinaryExpr<Bool>;
pub type BoolUnary = UnaryExpr<Bool>;

// A variable of type Bool. Holds its name, value, and Felt representation.
#[derive(Clone, Debug, Default)]
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
        let mut self_copy = self.clone();
        self_copy.as_felt = FeltExpr::default();
        self.as_felt
            .set_parent(BoolExpr::Var(self_copy).into(), None);
    }
}

#[derive(Clone, Debug)]
pub enum BoolExpr {
    Var(BoolVar),
    Binary(BoolBinary),
    Unary(BoolUnary),
}

impl BoolExpr {
    pub fn as_felt(&mut self) -> &mut FeltExpr {
        match self {
            BoolExpr::Var(v) => &mut v.as_felt,
            BoolExpr::Unary(u) => {
                if u.op == UnaryOp::BoolFromFelt {
                    if let GenericAirVar::Expr(ExprImpl::Felt(felt_expr)) = &mut *u.child {
                        return felt_expr;
                    }
                }
                panic!("Cannot convert to a Felt");
            }
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    // Converts a constant BoolExpr to a FeltExpr.
    pub fn const_to_felt(&self) -> FeltExpr {
        assert!(self.is_const());

        let value = self.value().map(|c| c.as_felt());
        FeltExpr::Unary(UnaryExpr::new(
            UnaryOp::ConstBoolToFelt,
            self.clone().into(),
            value,
        ))
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
                "as_felt".to_string(),
                value.map(|v| v.as_felt()),
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
            BoolExpr::Binary(b) => b.value,
            BoolExpr::Unary(u) => u.value,
        }
    }
}

impl AirVar for BoolExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn name(&self) -> String {
        match self {
            BoolExpr::Var(v) => v.name.clone(),
            BoolExpr::Binary(b) => b.name.clone(),
            BoolExpr::Unary(u) => u.name.clone(),
        }
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

    fn in_state(&self) -> bool {
        match self {
            BoolExpr::Var(v) => v.as_felt.in_state(),
            BoolExpr::Binary(b) => b.left.in_state() && b.right.in_state(),
            BoolExpr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt()]
    }

    fn is_const(&self) -> bool {
        match self {
            BoolExpr::Var(v) => v.is_const,
            BoolExpr::Binary(b) => b.left.is_const() && b.right.is_const(),
            BoolExpr::Unary(u) => u.child.is_const(),
        }
    }
}

impl Default for BoolExpr {
    fn default() -> Self {
        BoolExpr::Var(BoolVar::default())
    }
}

impl From<BoolVar> for BoolExpr {
    fn from(v: BoolVar) -> BoolExpr {
        BoolExpr::Var(v)
    }
}

impl From<BoolBinary> for BoolExpr {
    fn from(b: BoolBinary) -> BoolExpr {
        BoolExpr::Binary(b)
    }
}

impl From<BoolUnary> for BoolExpr {
    fn from(u: BoolUnary) -> BoolExpr {
        BoolExpr::Unary(u)
    }
}

impl From<BoolExpr> for GenericAirVar {
    fn from(expr: BoolExpr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<BoolExpr> for ProcessedAirVar {
    fn from(expr: BoolExpr) -> ProcessedAirVar {
        match expr {
            BoolExpr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return ProcessedAirVar::Var(Bool::r#type(), v.name);
                }
                if v.is_const {
                    return ProcessedAirVar::Const(Bool::r#type(), v.value.unwrap().calc());
                }
                ProcessedAirVar::Var(Bool::r#type(), v.name)
            }
            BoolExpr::Binary(b) => b.into(),
            BoolExpr::Unary(u) => u.into(),
        }
    }
}

#[macro_export]
macro_rules! const_bool_expr {
    ($val:expr) => {
        BoolExpr::new_const(Bool { value: $val })
    };
}

#[macro_export]
macro_rules! bool_expr {
    ($name:expr, $val:expr) => {
        BoolExpr::new_var($name.to_string(), Some(Bool::from($val)), None, false)
    };
}
