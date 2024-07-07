use std::array::from_fn;
use std::fmt::{Debug, Display};

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use super::autogen_structs::*;
use super::expressions::bool_expr::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::expressions::uint16_expr::*;
use super::expressions::uint32_expr::*;

#[cfg(test)]
use super::prover_types::*;

// Macros
use crate::impl_air_var;

/// Every input and output of an air function is an AirVar.
pub trait AirVar: Clone + Debug + Into<GenericAirVar> {
    fn new(name: String) -> Self;
    fn let_for_deduction(&self, name: String) -> Self;
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    // An AirVar is in_state if it is stored in a trace cell or a polynomial of felts stored in trace cells.
    // Used to verify that expressions of constraints are polynomials of felts written to the trace.
    // We check this in run mode, since when building an air body, we want all constraints to refer to sepecial
    // inputs carrying the AirFn name.
    fn in_state(&self) -> bool;
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
    // An AirVar is_const if was created with a value and the flag is_const = true, or if it is the result of
    // operations on other constants.
    // Used to verify that a constant variable is not written to the trace in a top-level AirFn, since this
    // would create a constant column in the trace.
    // Note that in runtime, we allow deduction of constant variables in internal calls, since an AirFn can
    // be called with different inputs in different calls.
    fn is_const(&self) -> bool;
}

// Air variables as represented in the air_body.
#[derive(Clone, Debug)]
pub enum GenericAirVar {
    Expr(ExprImpl),
    Tuple(Vec<GenericAirVar>),
    Array(Vec<GenericAirVar>),
}

impl GenericAirVar {
    pub fn in_state(&self) -> bool {
        match self {
            GenericAirVar::Expr(expr) => expr.in_state(),
            GenericAirVar::Tuple(vars) => vars.iter().all(|v| v.in_state()),
            GenericAirVar::Array(vars) => vars.iter().all(|v| v.in_state()),
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            GenericAirVar::Expr(expr) => expr.is_const(),
            GenericAirVar::Tuple(vars) => vars.iter().all(|v| v.is_const()),
            GenericAirVar::Array(vars) => vars.iter().all(|v| v.is_const()),
        }
    }
}

impl Serialize for GenericAirVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GenericAirVar::Expr(expr) => {
                let var: ProcessedAirVar = expr.clone().into();
                serializer.collect_str(&format!("{} (type: {})", var, expr.r#type()))
            }
            GenericAirVar::Tuple(vars) | GenericAirVar::Array(vars) => {
                let mut seq = serializer.serialize_seq(Some(vars.len()))?;
                for var in vars {
                    seq.serialize_element(var)?;
                }
                seq.end()
            }
        }
    }
}

impl Default for GenericAirVar {
    fn default() -> Self {
        GenericAirVar::Expr(ExprImpl::default())
    }
}

impl Display for GenericAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericAirVar::Expr(expr) => {
                let var: ProcessedAirVar = expr.clone().into();
                write!(f, "{}", var)
            }
            GenericAirVar::Tuple(vars) => {
                write!(f, "(")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ")")
            }
            GenericAirVar::Array(vars) => {
                write!(f, "[")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl From<GenericAirVar> for ProcessedAirVar {
    fn from(generic: GenericAirVar) -> ProcessedAirVar {
        match generic {
            GenericAirVar::Expr(expr) => expr.into(),
            GenericAirVar::Tuple(v) => {
                ProcessedAirVar::Tuple(v.into_iter().map(|v| v.into()).collect())
            }
            GenericAirVar::Array(v) => {
                ProcessedAirVar::Array(v.into_iter().map(|v| v.into()).collect())
            }
        }
    }
}

impl From<()> for GenericAirVar {
    fn from(_value: ()) -> Self {
        GenericAirVar::Tuple(vec![])
    }
}

impl AirVar for () {
    fn new(_name: String) -> Self {}

    fn let_for_deduction(&self, _name: String) -> Self {}

    fn name(&self) -> String {
        "()".to_string()
    }

    fn in_state(&self) -> bool {
        true
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![]
    }

    fn is_const(&self) -> bool {
        true
    }
}

impl_air_var!((BoolExpr, FeltExpr));
impl_air_var!((BoolExpr, UInt16Expr));
impl_air_var!((UInt16Expr, FeltExpr));
impl_air_var!([UInt32Expr]);
impl_air_var!([BoolExpr]);
impl_air_var!([FeltExpr]);
impl_air_var!([UInt16Expr]);

// Implements AirVar for arrays and tuples of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [$s:ty] ) => {
        impl<const N:usize> AirVar for [$s;N] where $s: AirVar
        {
            fn name(&self) -> String {
                format!("[{}]", self.iter().map(|s| s.name()).collect::<Vec<String>>().join(", "))
            }
            fn in_state(&self) -> bool {
                self.iter().all(|s| s.in_state())
            }
            fn is_const(&self) -> bool {
                self.iter().all(|s| s.is_const())
            }
            fn let_for_deduction(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_for_deduction(format!("{}[{}]", name, i));
                }
                res
            }
            fn new(name: String) -> Self {
                from_fn(|i| <$s as AirVar>::new(format!("{}[{}]", name, i)))
            }
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts_mut()).collect()
            }
        }

        impl<const N:usize> From<[$s;N]> for GenericAirVar {
            fn from(array: [$s;N]) -> GenericAirVar {
                GenericAirVar::Array(array.into_iter().map(|s| s.into()).collect())
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
            fn let_for_deduction(&self, name: String) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.let_for_deduction(format!("{}.{}", name, { i += 1; i - 1 })),)+)
            }
            fn new(name: String) -> Self {
                let mut i = 0;
                ($(<$s as AirVar>::new(format!("{}.{}", name, { i += 1; i - 1 })),)+)
            }
            fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
                let mut res = vec!();
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $(res.extend($s.as_felts_mut());)+
                res
            }
        }

        impl From<($($s),+)> for GenericAirVar {
            fn from(tuple: ($($s),+)) -> GenericAirVar {
                #[allow(non_snake_case)]
                let ($($s),+) = tuple.clone();
                GenericAirVar::Tuple(vec![$($s.into(),)+])
            }
        }
    };
}
