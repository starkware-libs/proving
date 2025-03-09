use std::collections::HashSet;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::super::air_body::*;
use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use crate::core::Felt;

pub trait VarExprUpdate {
    fn create_children(&mut self);
    fn update_children(&mut self);
}

#[derive(Clone, Debug)]
pub struct VarExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    pub(super) value: Option<T>,
    pub(super) is_const: bool,
    pub(super) parent: Option<ParentExpr>,
    pub(super) complex_or_felt: ComplexOrFelt,
    pub(super) is_deduction_intermediate: bool,
}

impl<T> VarExpr<T>
where
    T: ProverType,
{
    pub fn new(name: String, value: Option<T>, is_const: bool, in_state: bool) -> Self
    where
        Self: VarExprUpdate,
    {
        if is_const {
            assert!(value.is_some());
        }

        let mut var = VarExpr {
            name,
            value,
            is_const,
            parent: None,
            complex_or_felt: ComplexOrFelt::Felt(StateInfo::IsPolyOfState(in_state)),
            is_deduction_intermediate: false,
        };
        var.create_children();
        var.update_children();
        var
    }

    pub fn new_const(value: T) -> Self
    where
        Self: VarExprUpdate,
    {
        Self::new(value.calc(), Some(value), true, false)
    }

    pub(super) fn set_parent<P>(&mut self, parent_var: &VarExpr<P>, index: Option<usize>)
    where
        P: ProverType,
        Self: VarExprUpdate,
    {
        let parent = ParentExpr {
            name: parent_var.name.clone(),
            r#type: P::r#type(),
            parent: parent_var.parent.clone().map(Box::new),
            index,
            child_name: self.name.clone(),
            is_deduction_intermediate: parent_var.is_deduction_intermediate,
        };

        self.parent = Some(parent);
        self.update_children();
    }

    // Variable is directly in state if it's written to the state (has a state index), in an
    // external state (a preprocessed column), a public param or a const felt.
    pub fn is_directly_in_state(&self) -> bool {
        if self.is_const && T::r#type() == Felt::r#type() {
            return true;
        }

        matches!(
            self.complex_or_felt,
            ComplexOrFelt::Felt(StateInfo::StateIndex(..))
                | ComplexOrFelt::Felt(StateInfo::ExtTableState { .. })
                | ComplexOrFelt::Felt(StateInfo::PublicParam(_))
        )
    }

    pub fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        // self is a constant
        if self.is_const {
            return CompiledAirVar::Const(
                T::r#type(),
                self.value.expect("Const must have a value").calc(),
            );
        }

        // self was written to the trace
        if let ComplexOrFelt::Felt(StateInfo::StateIndex(i, desc)) = self.complex_or_felt {
            return CompiledAirVar::State(State::get_cell_name(i, &desc));
        }

        // self was written to the trace of an external const table
        if let ComplexOrFelt::Felt(StateInfo::ExtTableState(name, args)) = self.complex_or_felt {
            return CompiledAirVar::ExternalState(name, args);
        }

        // self is a public param
        if let ComplexOrFelt::Felt(StateInfo::PublicParam(param)) = self.complex_or_felt {
            return CompiledAirVar::PublicParam(param.name());
        }

        if compile_for == CompileFor::Deductions {
            // self is an intermediate visible in deductions
            if self.is_deduction_intermediate {
                return CompiledAirVar::Var(T::r#type(), self.name);
            }
        } else {
            // <compile_for> == CompileFor::Constraints
            // self is an intermediate visible in constraints
            if let ComplexOrFelt::Felt(StateInfo::ConstraintIntermediate(name)) =
                self.complex_or_felt
            {
                return CompiledAirVar::Var(T::r#type(), name);
            }
        }

        // self is a field of another variable
        if let Some(parent) = self.parent {
            return parent.get_compiled_child();
        }

        // self is a standalone variable
        CompiledAirVar::Var(T::r#type(), self.name)
    }
}

impl<T> AsProverType<T> for VarExpr<T>
where
    T: ProverType,
{
    fn value(&self) -> Option<T> {
        self.value
    }
}

impl<T> InternalAirVarInfo for VarExpr<T>
where
    T: ProverType,
{
    fn get_info(&self) -> HashSet<AirVarInfo> {
        let in_state = if self.is_directly_in_state() {
            true
        } else {
            match &self.complex_or_felt {
                ComplexOrFelt::Felt(StateInfo::IsPolyOfState(b)) => *b,
                ComplexOrFelt::Felt(StateInfo::ConstraintIntermediate(_)) => true,
                ComplexOrFelt::Complex(children) => children.iter().all(|c| c.in_state()),
                _ => unreachable!("Other cases of complex_or_felt are directly in state"),
            }
        };

        let is_constraint_intermediate = matches!(
            self.complex_or_felt,
            ComplexOrFelt::Felt(StateInfo::ConstraintIntermediate(_))
        );

        let visibility = Visibility {
            // A variable is visible in deductions if it's an intermediate in deductions, has no
            // intermediates, or if it has a parent (since all parents are visibile in deductions)
            in_deductions: self.is_deduction_intermediate
                || self.is_directly_in_state()
                || self.parent.is_some()
                || !is_constraint_intermediate,
            // A variables is visible in constraints if it's an intermediate in constraints, or if
            // it's directly in state
            in_constraints: is_constraint_intermediate || self.is_directly_in_state(),
        };

        let info = AirVarInfo {
            in_state,
            is_const: self.is_const,
            visibility,
            public_param: if let ComplexOrFelt::Felt(StateInfo::PublicParam(ref p)) =
                self.complex_or_felt
            {
                Some(p.clone())
            } else {
                None
            },
            external_state: if let ComplexOrFelt::Felt(StateInfo::ExtTableState(name, args)) =
                self.complex_or_felt.clone()
            {
                Some((name, args))
            } else {
                None
            },
        };
        HashSet::from([info])
    }

    fn prover_type(&self) -> String {
        T::r#type()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ParentExpr {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) parent: Option<Box<ParentExpr>>,
    pub(super) index: Option<usize>,
    pub(super) child_name: String,
    pub(super) is_deduction_intermediate: bool,
}

impl ParentExpr {
    pub(super) fn get_compiled_child(self) -> CompiledAirVar {
        let args = if let Some(i) = self.index {
            let index_var = CompiledAirVar::Const("usize".to_string(), i.to_string());
            vec![index_var]
        } else {
            vec![]
        };

        CompiledAirVar::MethodCall(Box::new(self.clone().into()), self.child_name, args)
    }
}

impl From<ParentExpr> for CompiledAirVar {
    fn from(expr: ParentExpr) -> CompiledAirVar {
        if expr.is_deduction_intermediate {
            return CompiledAirVar::Var(expr.r#type, expr.name);
        }

        if let Some(parent) = expr.parent {
            return parent.get_compiled_child();
        }

        CompiledAirVar::Var(expr.r#type, expr.name)
    }
}

// Each VarExpr is either a single felt, that can be written to the state and an intermediate in
// constraints, or a complex expression that holds one or more expressions (children).
#[derive(Clone, Debug)]
pub(super) enum ComplexOrFelt {
    Complex(Vec<ExprImpl>),
    Felt(StateInfo),
}

impl ComplexOrFelt {
    pub(super) fn as_complex_mut(&mut self) -> &mut [ExprImpl] {
        if let ComplexOrFelt::Complex(children) = self {
            return children;
        }
        panic!("Expected complex expression");
    }

    pub(super) fn as_complex(&self) -> &[ExprImpl] {
        if let ComplexOrFelt::Complex(children) = self {
            return children;
        }
        panic!("Expected complex expression");
    }
}
