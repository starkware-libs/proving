use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;

pub type UInt32Operation = OpExpr<UInt32>;
// A variable of type UInt32. Holds its name, and value. It is represented as two UInt16 variables.
#[derive(Clone, Debug, Default)]
pub struct UInt32Var {
    pub(super) name: String,
    pub(super) value: Option<UInt32>,
    pub(super) low: UInt16Expr,
    pub(super) high: UInt16Expr,
    pub(super) parent: Option<Box<ExprImpl>>,
    pub(super) is_const: bool,
}

impl UInt32Var {
    // Updates the low and high parts of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        let mut self_copy = self.clone();
        self_copy.low = UInt16Expr::default();
        self_copy.high = UInt16Expr::default();
        let parent: ExprImpl = UInt32Expr::Var(self_copy.clone()).into();
        self.low.set_parent(parent.clone());
        self.high.set_parent(parent);
    }
}

#[derive(Clone, Debug)]
pub enum UInt32Expr {
    Var(UInt32Var),
    Op(UInt32Operation),
}

impl UInt32Expr {
    pub fn low_mut(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.low,
            _ => panic!("Cannot convert non-variable to UInt16"),
        }
    }

    pub fn high_mut(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.high,
            _ => panic!("Cannot convert non-variable to UInt16"),
        }
    }

    pub fn low(&self) -> UInt16Expr {
        self.clone().low_mut().clone()
    }

    pub fn high(&self) -> UInt16Expr {
        self.clone().high_mut().clone()
    }

    // Called whenever a parent variable is created (see update_parts of UInt64Expr).
    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let UInt32Expr::Var(v) = self {
            v.parent = Some(Box::new(parent));
            v.update_parts();
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new UInt32Var.
    pub fn new_var(
        name: String,
        value: Option<UInt32>,
        low_state_index: Option<usize>,
        high_state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = UInt32Var {
            name,
            value,
            low: UInt16Expr::new_var(
                "low".to_string(),
                value.map(|v| v.low()),
                low_state_index,
                is_const,
            ),
            high: UInt16Expr::new_var(
                "high".to_string(),
                value.map(|v| v.high()),
                high_state_index,
                is_const,
            ),
            parent: None,
            is_const,
        };
        res.update_parts();
        res.into()
    }

    // Creates a new constant UInt32Var.
    pub fn new_const(value: UInt32) -> Self {
        Self::new_var(value.calc(), Some(value), None, None, true)
    }
}

impl Expr<UInt32> for UInt32Expr {
    fn value(&self) -> Option<UInt32> {
        match self {
            UInt32Expr::Var(v) => v.value,
            UInt32Expr::Op(op) => op.value,
        }
    }
}

impl AirVar for UInt32Expr {
    fn name(&self) -> String {
        match self {
            UInt32Expr::Var(v) => v.name.clone(),
            UInt32Expr::Op(op) => op.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            UInt32Expr::Var(v) => vec![v.low.as_felt_mut(), v.high.as_felt_mut()],
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }
}

impl InternalAirVarActions for UInt32Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, None, false)
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt32Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for UInt32Expr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            UInt32Expr::Var(v) => v.low.in_state() && v.high.in_state(),
            UInt32Expr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            UInt32Expr::Var(v) => v.is_const,
            UInt32Expr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

impl Default for UInt32Expr {
    fn default() -> Self {
        UInt32Expr::Var(UInt32Var::default())
    }
}

impl From<UInt32Var> for UInt32Expr {
    fn from(v: UInt32Var) -> UInt32Expr {
        UInt32Expr::Var(v)
    }
}

impl From<UInt32Operation> for UInt32Expr {
    fn from(b: UInt32Operation) -> UInt32Expr {
        UInt32Expr::Op(b)
    }
}

impl From<UInt32Expr> for CompiledAirVar {
    fn from(expr: UInt32Expr) -> CompiledAirVar {
        match expr {
            UInt32Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return CompiledAirVar::Var(UInt32::r#type(), v.name);
                }
                if v.is_const {
                    return CompiledAirVar::Const(UInt32::r#type(), v.value.unwrap().calc());
                }
                if let Some(var) = v.parent {
                    return CompiledAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }

                CompiledAirVar::Var(UInt32::r#type(), v.name)
            }
            UInt32Expr::Op(op) => op.into(),
        }
    }
}

#[macro_export]
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Expr::new_const($val.into())
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::new_var(
            $name.to_string(),
            Some(UInt32::from($val)),
            None,
            None,
            false,
        )
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            UInt32Expr::new_var(
                $name.to_string(),
                Some(UInt32::from($val)),
                Some(0),
                Some(1),
                false,
            )
        } else {
            UInt32Expr::new_var(
                $name.to_string(),
                Some(UInt32::from($val)),
                None,
                None,
                false,
            )
        }
    };
}
