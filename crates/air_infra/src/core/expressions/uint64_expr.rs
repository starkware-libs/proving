use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint32_expr::*;

pub type UInt64Operation = OpExpr<UInt64>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

// A variable of type UInt64. Holds its name, and value. It is represented as two UInt32 variables.
#[derive(Clone, Debug)]
pub struct UInt64Var {
    pub(super) name: String,
    pub(super) value: Option<UInt64>,
    pub(super) low: UInt32Expr,
    pub(super) high: UInt32Expr,
    pub(super) is_const: bool,
}

impl UInt64Var {
    // Updates the low and high parts of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        self.low.set_parent(ParentExpr {
            name: self.name.clone(),
            r#type: UInt64::r#type(),
            parent: None,
            index: None,
            child_name: LOW_NAME.to_string(),
        });
        self.high.set_parent(ParentExpr {
            name: self.name.clone(),
            r#type: UInt64::r#type(),
            parent: None,
            index: None,
            child_name: HIGH_NAME.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub enum UInt64Expr {
    Var(UInt64Var),
    Op(UInt64Operation),
}

impl UInt64Expr {
    pub fn low_mut(&mut self) -> &mut UInt32Expr {
        match self {
            UInt64Expr::Var(v) => &mut v.low,
            _ => panic!("Cannot convert non-variable to UInt32"),
        }
    }

    pub fn high_mut(&mut self) -> &mut UInt32Expr {
        match self {
            UInt64Expr::Var(v) => &mut v.high,
            _ => panic!("Cannot convert non-variable to UInt32"),
        }
    }

    pub fn low(&self) -> UInt32Expr {
        self.clone().low_mut().clone()
    }

    pub fn high(&self) -> UInt32Expr {
        self.clone().high_mut().clone()
    }

    // Creates a new UInt64Var.
    pub fn new_var(
        name: String,
        value: Option<UInt64>,
        ll_state_index: Option<usize>,
        lh_state_index: Option<usize>,
        hl_state_index: Option<usize>,
        hh_state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = UInt64Var {
            name,
            value,
            low: UInt32Expr::new_var(
                LOW_NAME.to_string(),
                value.map(|v| v.low()),
                ll_state_index,
                lh_state_index,
                is_const,
            ),
            high: UInt32Expr::new_var(
                HIGH_NAME.to_string(),
                value.map(|v| v.high()),
                hl_state_index,
                hh_state_index,
                is_const,
            ),
            is_const,
        };
        res.update_parts();
        res.into()
    }

    // Creates a new constant UInt64Var.
    pub fn new_const(value: UInt64) -> Self {
        Self::new_var(value.calc(), Some(value), None, None, None, None, true)
    }
}

impl Expr<UInt64> for UInt64Expr {
    fn value(&self) -> Option<UInt64> {
        match self {
            UInt64Expr::Var(v) => v.value,
            UInt64Expr::Op(op) => op.value,
        }
    }
}

impl AirVar for UInt64Expr {
    fn name(&self) -> String {
        match self {
            UInt64Expr::Var(v) => v.name.clone(),
            UInt64Expr::Op(op) => op.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            UInt64Expr::Var(v) => {
                let mut res = vec![];
                res.append(&mut v.low.as_felts_mut());
                res.append(&mut v.high.as_felts_mut());
                res
            }
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }
}

impl InternalAirVarActions for UInt64Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, None, None, None, false)
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt64Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, None, None, None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for UInt64Expr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            UInt64Expr::Var(v) => v.low.in_state() && v.high.in_state(),
            UInt64Expr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            UInt64Expr::Var(v) => v.is_const,
            UInt64Expr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

impl From<UInt64Var> for UInt64Expr {
    fn from(v: UInt64Var) -> UInt64Expr {
        UInt64Expr::Var(v)
    }
}

impl From<UInt64Operation> for UInt64Expr {
    fn from(b: UInt64Operation) -> UInt64Expr {
        UInt64Expr::Op(b)
    }
}

impl From<UInt64Expr> for CompiledAirVar {
    fn from(expr: UInt64Expr) -> CompiledAirVar {
        match expr {
            UInt64Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return CompiledAirVar::Var(UInt64::r#type(), v.name);
                }
                if v.is_const {
                    return CompiledAirVar::Const(UInt64::r#type(), v.value.unwrap().calc());
                }
                CompiledAirVar::Var(UInt64::r#type(), v.name)
            }
            UInt64Expr::Op(op) => op.into(),
        }
    }
}

#[macro_export]
macro_rules! const_u64_expr {
    ($val:expr) => {
        UInt64Expr::new_const($val.into())
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! u64_expr {
    ($name:expr, $val:expr) => {
        UInt64Expr::new_var(
            $name.to_string(),
            Some(UInt64::from($val)),
            None,
            None,
            None,
            None,
            false,
        )
    };
}
