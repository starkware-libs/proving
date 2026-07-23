use std::iter::Sum;

use air_common::ExternalState;
use indexmap::IndexSet;

use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;
use super::var_expr::*;
// Macros
use crate::const_expr;
use crate::core::Felt;
use crate::core::public_params::PublicParam;

pub type FeltOperation = OpExpr<Felt>;
pub type FeltExpr = Expr<Felt>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeltInfo {
    pub value_info: ValueInfo,
    // If some, the felt is an intermediate variable used in constraints.
    pub constraint_intermediate: Option<String>,
    pub is_const: bool,
}

impl FeltInfo {
    // Checks if this felt will be compiled into a constraint intermediate. This requires
    // 1. That the felt is stored in a constraint intermediate, and
    // 2. That the felt is not stored in a trace cell / public parameter, because in these cases the
    //    compilation will prefer to compile it directly as CompiledAirVar::State or ::PublicParam.
    pub fn get_used_constraint_intermediate_name(&self) -> Option<String> {
        if let Some(ref name) = self.constraint_intermediate
            && matches!(self.value_info, ValueInfo::DegPolyOfState(_))
        {
            return Some(name.clone());
        }
        None
    }

    // If this felt is stored in a state cell, return the name of that cell in the
    // compiled AirFn.
    pub fn get_state_cell_name(&self) -> Option<String> {
        if let ValueInfo::StateIndex(index, ref desc) = self.value_info {
            Some(State::get_cell_name(index, desc))
        } else {
            None
        }
    }
}

// Describes the value of a felt var
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueInfo {
    // The felt is in the state of the current component, at the specified index.
    // The second argument is the description of the trace cell. It is used only for compilation.
    // Consider moving to a compilation context.
    StateIndex(usize, Option<String>),
    // If the <Option> value is Some(<deg>), the felt is a polynomial expression in the state, of
    // degree at most <deg>. If the <Option> is None, the felt is not a
    // polynomial expression in the state (for example, a value read from the memory and not
    // written to the state yet).
    DegPolyOfState(Option<usize>),
    // The felt is in  an external state (a preprocessed column).
    ExternalState(ExternalState),
    // The felt is one of the public parameters.
    PublicParam(PublicParam),
    // The felt is 1 in active rows and 0 in padding rows
    Enabler,
    // The felt is the multiplicity of the i-th relation of the component
    Multiplicity(usize),
}

impl VarExprUpdate for VarExpr<Felt> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        self.complex_or_felt = ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::DegPolyOfState(deg_in_state),
            constraint_intermediate: None,
            is_const,
        });
    }
    fn update_children(&mut self) {
        // Felt does not have children.
    }
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change its felts
    // into variables that have state information.
    pub fn set_value(&mut self, new_value_info: ValueInfo) {
        let name = match &new_value_info {
            ValueInfo::StateIndex(index, desc) => State::get_cell_name(*index, desc),
            ValueInfo::DegPolyOfState(_) => {
                panic!("set_value shouldn't be used to make a FeltExpr a DegPolyOfState")
            }
            ValueInfo::ExternalState(col_id) => col_id.clone(),
            ValueInfo::PublicParam(public_param) => public_param.name(),
            ValueInfo::Enabler => "enabler".to_string(),
            ValueInfo::Multiplicity(idx) => format!("multiplicity_{idx}"),
        };

        match self {
            FeltExpr::Var(v) => {
                v.name = name;
            }
            _ => {
                *self = Expr::new_var_from(name, self);
            }
        }
        self.as_var_mut().complex_or_felt.as_felt_info_mut().value_info = new_value_info;
        self.as_var_mut().visibility.in_constraints = true;
        self.as_var_mut().visibility.in_deductions = true;
    }

    // Felt is directly in state if it's written to the state (has a state index), in an external
    // state (a preprocessed column), a public param, or a const felt.
    pub fn is_directly_in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            FeltExpr::Var(v) => matches!(
                v.complex_or_felt.as_felt_info().value_info,
                ValueInfo::StateIndex(..)
                    | ValueInfo::ExternalState { .. }
                    | ValueInfo::PublicParam(_)
            ),
            _ => false,
        }
    }

    /// Return the `FeltInfo` from the leaves of this `FeltExpr`
    pub fn var_infos(&self) -> Vec<&FeltInfo> {
        let mut result: Vec<&FeltInfo> = vec![];
        match self {
            Expr::Var(var_expr) => result.push(var_expr.complex_or_felt.as_felt_info()),
            Expr::Op(op_expr) => {
                for child in op_expr.children.iter() {
                    let AirVarImpl::Expr(ExprImpl::Felt(felt_expr)) = child else {
                        panic!("Unexpected child {child:?} in FeltExpr::Op")
                    };
                    result.extend(felt_expr.var_infos());
                }
            }
        }
        result
    }

    /// Return the set of intermediate values that the constraint evaluation
    /// code for this expression will access.
    pub fn get_used_constraint_intermediates(&self) -> IndexSet<String> {
        self.var_infos()
            .iter()
            .filter_map(|vi| vi.get_used_constraint_intermediate_name())
            .collect()
    }

    pub fn let_for_constraint(&mut self, name: String) {
        if let FeltExpr::Op(_) = self {
            let mut var = Expr::new_var_from(name.clone(), self);
            var.as_var_mut().visibility.in_deductions = false;
            *self = var;
        }
        self.as_var_mut().complex_or_felt.as_felt_info_mut().constraint_intermediate = Some(name);
        self.as_var_mut().visibility.in_constraints = true;
    }
}

impl TryIntoFeltExpr for FeltExpr {
    fn try_into_felt(&mut self) -> Option<&mut FeltExpr> {
        Some(self)
    }
}

// Default is implemented for FeltExpr because it is stored in memory.
impl Default for FeltExpr {
    fn default() -> Self {
        const_expr!(0)
    }
}

impl Sum for FeltExpr {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(const_expr!(0), |sum, f| sum + f)
    }
}

#[macro_export]
macro_rules! const_expr {
    ($val:expr) => {
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $crate::core::Felt::from($val as u32),
        ))
    };
}

#[macro_export]
macro_rules! const_expr_from_m31 {
    ($val:expr) => {
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new_const($val))
    };
}

#[cfg(test)]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some($crate::core::Felt::from($val)),
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(in crate::core) use expr;
