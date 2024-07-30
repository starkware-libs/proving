use std::array::from_fn;
use std::fmt::{Debug, Display};

use enum_dispatch::enum_dispatch;
use serde::Serialize;

use super::compiled_structs::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt252_expr::*;
use super::expressions::felt_expr::*;
use super::expressions::uint16_expr::*;
use super::expressions::uint32_expr::*;
use super::expressions::uint64_expr::*;

#[cfg(test)]
use super::Felt;

// Macros
use crate::impl_air_var;

/// Every input and output of an air function is an AirVar.
pub trait AirVar: InternalAirVarInfo + InternalAirVarActions {
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr>;
    fn as_felts(&self) -> Vec<FeltExpr> {
        self.clone()
            .as_felts_mut()
            .into_iter()
            .map(|f| f.clone())
            .collect()
    }
    #[cfg(test)]
    fn to_values(&self) -> Vec<Felt> {
        self.as_felts().iter().map(|f| f.value().unwrap()).collect()
    }
}

// Information about air variables used by the air builder.
#[enum_dispatch]
pub trait InternalAirVarInfo: Debug {
    // An AirVar is in_state if it is stored in a trace cell or a polynomial of felts stored in trace cells.
    // Used to verify that expressions of constraints are polynomials of felts written to the trace.
    // We check this in run mode, since when building an air body, we want all constraints to refer to sepecial
    // inputs carrying the AirFn name.
    fn in_state(&self) -> bool;

    // An AirVar is_const if was created with a value and the flag is_const = true, or if it is the result of
    // operations on other constants.
    // Used to verify that a constant variable is not written to the trace in a top-level AirFn, since this
    // would create a constant column in the trace.
    // Note that in runtime, we allow deduction of constant variables in internal calls, since an AirFn can
    // be called with different inputs in different calls.
    fn is_const(&self) -> bool;
}

// Actions on air variables used by the air builder.
pub trait InternalAirVarActions: Clone + Into<AirVarImpl> {
    fn new(name: String) -> Self;
    fn let_for_deduction(&self, name: String) -> Self;
}

// Air variables as represented in the air_body.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AirVarImpl {
    Expr(ExprImpl),
    Tuple(Vec<AirVarImpl>),
    Array(Vec<AirVarImpl>),
}

impl InternalAirVarInfo for AirVarImpl {
    fn in_state(&self) -> bool {
        match self {
            AirVarImpl::Expr(expr) => expr.in_state(),
            AirVarImpl::Tuple(vars) => vars.iter().all(|v| v.in_state()),
            AirVarImpl::Array(vars) => vars.iter().all(|v| v.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            AirVarImpl::Expr(expr) => expr.is_const(),
            AirVarImpl::Tuple(vars) => vars.iter().all(|v| v.is_const()),
            AirVarImpl::Array(vars) => vars.iter().all(|v| v.is_const()),
        }
    }
}

impl Display for AirVarImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirVarImpl::Expr(expr) => {
                write!(f, "{}", expr)
            }
            AirVarImpl::Tuple(_) | AirVarImpl::Array(_) => {
                write!(f, "{}", CompiledAirVar::from(self.clone()))
            }
        }
    }
}

impl From<AirVarImpl> for CompiledAirVar {
    fn from(generic: AirVarImpl) -> CompiledAirVar {
        match generic {
            AirVarImpl::Expr(expr) => expr.into(),
            AirVarImpl::Tuple(v) => {
                CompiledAirVar::Tuple(v.into_iter().map(|v| v.into()).collect())
            }
            AirVarImpl::Array(v) => {
                CompiledAirVar::Array(v.into_iter().map(|v| v.into()).collect())
            }
        }
    }
}

impl From<()> for AirVarImpl {
    fn from(_value: ()) -> Self {
        AirVarImpl::Tuple(vec![])
    }
}

impl AirVar for () {
    fn name(&self) -> String {
        "()".to_string()
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![]
    }
}

impl InternalAirVarInfo for () {
    fn in_state(&self) -> bool {
        true
    }

    fn is_const(&self) -> bool {
        true
    }
}

impl InternalAirVarActions for () {
    fn new(_name: String) -> Self {}
    fn let_for_deduction(&self, _name: String) -> Self {}
}

impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((UInt16Expr, FeltExpr));
impl_air_var!([UInt32Expr]);
impl_air_var!([BoolExpr]);
impl_air_var!([FeltExpr]);
impl_air_var!([UInt16Expr]);
impl_air_var!([Felt252Expr]);
type Felts = [FeltExpr; 3];
type Bools = [BoolExpr; 15];
impl_air_var!((Felts, Bools));

// Implements AirVar for arrays and tuples of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [$s:ty] ) => {
        impl<const N:usize> AirVar for [$s;N] where $s: AirVar
        {
            fn name(&self) -> String {
                format!("[{}]", self.iter().map(|s| s.name()).collect::<Vec<String>>().join(", "))
            }
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
        }

        impl<const N:usize> InternalAirVarInfo for [$s;N] where $s: InternalAirVarInfo {
            fn in_state(&self) -> bool {
                self.iter().all(|s| s.in_state())
            }
            fn is_const(&self) -> bool {
                self.iter().all(|s| s.is_const())
            }
        }

        impl<const N:usize> InternalAirVarActions for [$s;N] where $s: InternalAirVarActions {
            fn let_for_deduction(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_for_deduction(format!("{}[{}]", name, i));
                }
                res
            }
            fn new(name: String) -> Self {
                from_fn(|i| <$s as InternalAirVarActions>::new(format!("{}[{}]", name, i)))
            }
        }

        impl<const N:usize> From<[$s;N]> for AirVarImpl {
            fn from(array: [$s;N]) -> AirVarImpl {
                AirVarImpl::Array(array.into_iter().map(|s| s.into()).collect())
            }
        }
    };

    (($($s:ident),+)) => {
        impl AirVar for ($($s),+) where $($s: AirVar),+
        {
            fn name(&self) -> String {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                format!("({})", vec![$($s.name(), )+].join(", "))
            }
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                let mut res = vec!();
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $(res.extend($s.as_felts_mut());)+
                res
            }
        }

        impl InternalAirVarInfo for ($($s),+) where $($s: InternalAirVarInfo),+
        {
            fn in_state(&self) -> bool {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $($s.in_state() &&)+ true
            }
            fn is_const(&self) -> bool {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $($s.is_const() &&)+ true
            }
        }

        impl InternalAirVarActions for ($($s),+) where $($s: InternalAirVarActions),+
        {
            fn let_for_deduction(&self, name: String) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.let_for_deduction(format!("{}.{}", name, { i += 1; i - 1 })),)+)
            }
            fn new(name: String) -> Self {
                let mut i = 0;
                ($(<$s as InternalAirVarActions>::new(format!("{}.{}", name, { i += 1; i - 1 })),)+)
            }
        }

        impl From<($($s),+)> for AirVarImpl {
            fn from(tuple: ($($s),+)) -> AirVarImpl {
                #[allow(non_snake_case)]
                let ($($s),+) = tuple.clone();
                AirVarImpl::Tuple(vec![$($s.into(),)+])
            }
        }
    };
}
