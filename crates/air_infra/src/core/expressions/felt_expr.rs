use compiled_casm_air::compiled_structs::ExternalState;
use compiled_casm_air::public_params::PublicParam;
use convert_case::{Case, Casing};
use indexmap::IndexSet;

use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;
use super::var_expr::*;
// Macros
use crate::const_expr;
use crate::core::Felt;

pub type FeltOperation = OpExpr<Felt>;
pub type FeltExpr = Expr<Felt>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeltInfo {
    pub state_info: StateInfo,
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
        if let Some(ref name) = self.constraint_intermediate {
            if matches!(self.state_info, StateInfo::DegPolyOfState(_)) {
                return Some(name.clone());
            }
        }
        None
    }

    // If this felt is stored in a state cell, return the name of that cell in the
    // compiled AirFn.
    pub fn get_state_cell_name(&self) -> Option<String> {
        if let StateInfo::StateIndex(index, ref desc) = self.state_info {
            Some(State::get_cell_name(index, desc))
        } else {
            None
        }
    }
}

// Describes where in the state this FeltExpr resides
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateInfo {
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
}

impl VarExprUpdate for VarExpr<Felt> {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>) {
        self.complex_or_felt = ComplexOrFelt::Felt(FeltInfo {
            state_info: StateInfo::DegPolyOfState(deg_in_state),
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
    pub fn to_state(&mut self, new_state_info: StateInfo) {
        let name = match &new_state_info {
            StateInfo::StateIndex(index, desc) => State::get_cell_name(*index, desc),
            StateInfo::DegPolyOfState(_) => {
                panic!("to_state shouldn't be used to make a FeltExpr an DegPolyOfState")
            }
            StateInfo::ExternalState(ExternalState {
                name,
                generic_param: _,
                args,
            }) => {
                format!("{}({})", name.to_case(Case::Snake), args.join(", "))
            }
            StateInfo::PublicParam(public_param) => public_param.name(),
        };

        match self {
            FeltExpr::Var(v) => {
                v.name = name;
            }
            _ => {
                *self = Expr::new_var_from(name, self);
            }
        }
        self.as_var_mut()
            .complex_or_felt
            .as_felt_info_mut()
            .state_info = new_state_info;
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
                v.complex_or_felt.as_felt_info().state_info,
                StateInfo::StateIndex(..)
                    | StateInfo::ExternalState { .. }
                    | StateInfo::PublicParam(_)
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
        self.as_var_mut()
            .complex_or_felt
            .as_felt_info_mut()
            .constraint_intermediate = Some(name);
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
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val,
        ))
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
