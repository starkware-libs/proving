use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use super::autogen_structs::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
#[cfg(test)]
use super::prover_types::*;

/// Every input and output of an air function is an AirVar.
pub trait AirVar: Clone + Debug + Default + Into<GenericAirVar> {
    fn new(name: String) -> Self;
    fn let_for_deduction(&self, name: String) -> Self;
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    // Returns whether the value of this AirVar is stored in a trace cell.
    // For example, an input to an air function is not in state when it is from the private input.
    fn in_state(&self) -> bool;
    fn as_felts(&mut self) -> Vec<&mut FeltExpr>;
    #[cfg(test)]
    fn to_values(&self) -> Vec<Felt> {
        self.clone()
            .as_felts()
            .into_iter()
            .map(|f| f.value().unwrap())
            .collect()
    }
}

// Air variables as represented in the air_body.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

impl Default for GenericAirVar {
    fn default() -> Self {
        GenericAirVar::Expr(ExprImpl::default())
    }
}

impl Display for GenericAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericAirVar::Expr(expr) => write!(f, "{}", expr),
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

    fn let_for_deduction(&self, name: String) -> Self {
        <Self as AirVar>::new(name)
    }

    fn name(&self) -> String {
        "()".to_string()
    }

    fn in_state(&self) -> bool {
        true
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![]
    }
}

// Implements AirVar for arrays and tuples of air vars.
#[macro_export]
macro_rules! impl_air_var {
    ( [$s:ty;$n:literal] ) => {
        impl AirVar for [$s;$n] where $s: AirVar
        {
            fn name(&self) -> String {
                format!("[{}]", self.iter().map(|s| s.name()).collect::<Vec<String>>().join(", "))
            }
            fn in_state(&self) -> bool {
                self.iter().all(|s| s.in_state())
            }
            fn let_for_deduction(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_for_deduction(format!("{}[{}]", name, i));
                }
                res
            }
            fn new(name: String) -> Self {
                from_fn(|i| <$s>::new(format!("{}[{}]", name, i)))
            }
            fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts()).collect()
            }
        }
        impl From<[$s;$n]> for GenericAirVar {
            fn from(array: [$s;$n]) -> GenericAirVar {
                GenericAirVar::Array(array.into_iter().map(|s| s.into()).collect())
            }
        }
    };

    ( Vec<$s:ty> ) => {
        impl AirVar for Vec<$s> where $s: AirVar
        {
            fn name(&self) -> String {
                format!("[{}]", self.iter().map(|s| s.name()).collect::<Vec<String>>().join(", "))
            }
            fn in_state(&self) -> bool {
                self.iter().all(|s| s.in_state())
            }
            fn let_for_deduction(&self, name: String) -> Self {
                let mut res = self.clone();
                for (i, s) in res.iter_mut().enumerate() {
                    *s = s.let_for_deduction(format!("{}[{}]", name, i));
                }
                res
            }
            fn new(_name: String) -> Self {
                panic!("Cannot create a new Vec AirVar with name");
            }
            fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
                self.into_iter().flat_map(|s| s.as_felts()).collect()
            }
        }
        impl From<Vec<$s>> for GenericAirVar {
            fn from(array: Vec<$s>) -> GenericAirVar {
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
            fn let_for_deduction(&self, name: String) -> Self {
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                let mut i = 0;
                ($($s.let_for_deduction(format!("{}.{}", name, { i += 1; i })),)+)
            }
            fn new(name: String) -> Self {
                let mut i = 0;
                ($(<$s>::new(format!("{}.{}", name, { i += 1; i })),)+)
            }
            fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
                let mut res = vec!();
                #[allow(non_snake_case)]
                let ($($s),+) = self;
                $(res.extend($s.as_felts());)+
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
