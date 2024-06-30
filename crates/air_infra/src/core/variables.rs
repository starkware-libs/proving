use std::fmt::{Debug, Display};

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

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

    fn is_const(&self) -> bool {
        true
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

    ( Option<$s:ty> ) => {
        impl AirVar for Option<$s> where $s: AirVar
        {
            fn name(&self) -> String {
                if self.is_some() {
                    format!("Some({})", self.as_ref().unwrap().name())
                } else {
                    "None".to_string()
                }
            }
            fn in_state(&self) -> bool {
                if self.is_some() {
                    self.as_ref().unwrap().in_state()
                } else {
                    true
                }
            }
            fn is_const(&self) -> bool {
                if self.is_some() {
                    self.as_ref().unwrap().is_const()
                } else {
                    true
                }
            }
            fn let_for_deduction(&self, name: String) -> Self {
                if self.is_some() {
                    Some(self.as_ref().unwrap().let_for_deduction(format!("{}", name)))
                } else {
                    panic!("Cannot let_for_deduction on None");
                }
            }
            fn new(_name: String) -> Self {
                panic!("Cannot create a new Option AirVar with name");
            }
            fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
                if self.is_some() {
                    self.as_mut().unwrap().as_felts()
                } else {
                    vec![]
                }
            }
        }
        impl From<Option<$s>> for GenericAirVar {
            fn from(o: Option<$s>) -> GenericAirVar {
                if o.is_some() {
                    GenericAirVar::from(o.unwrap())
                } else {
                    panic!("Cannot convert None to GenericAirVar");
                }
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
                ($(<$s>::new(format!("{}.{}", name, { i += 1; i - 1 })),)+)
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
