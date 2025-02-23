use std::collections::HashSet;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use crate::core::Felt;

pub trait VarExprUpdate {
    fn create_children(&mut self, in_deductions: bool, felts_in_constraints: bool);
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
    // Every variable can be <in_deductions>, but only felts can be <in_constraints>.
    pub(super) visibility: Visibility,
}

impl<T> VarExpr<T>
where
    T: ProverType,
    Self: VarExprUpdate,
{
    pub fn new(
        name: String,
        value: Option<T>,
        is_const: bool,
        in_state: bool,
        in_deductions: bool,
        felts_in_constraints: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut var = VarExpr {
            name,
            value,
            is_const,
            parent: None,
            complex_or_felt: ComplexOrFelt::Felt(StateInfo::IsPolyOfState(in_state)),
            // Only felts can have visibility in constraints.
            visibility: Visibility {
                in_deductions,
                in_constraints: felts_in_constraints && (T::r#type() == Felt::r#type()),
            },
        };
        var.create_children(in_deductions, felts_in_constraints);
        var.update_children();
        var
    }

    pub fn new_const(value: T) -> Self {
        Self::new(value.calc(), Some(value), true, false, true, true)
    }

    pub(super) fn set_parent<P>(&mut self, parent_var: &VarExpr<P>, index: Option<usize>)
    where
        P: ProverType,
    {
        let parent = ParentExpr {
            name: parent_var.name.clone(),
            r#type: P::r#type(),
            parent: parent_var.parent.clone().map(Box::new),
            index,
            child_name: self.name.clone(),
        };

        self.parent = Some(parent);
        self.update_children();
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
        let in_state = if self.is_const {
            true
        } else {
            match &self.complex_or_felt {
                ComplexOrFelt::Felt(StateInfo::StateIndex(..)) => true,
                ComplexOrFelt::Felt(StateInfo::IsPolyOfState(b)) => *b,
                ComplexOrFelt::Felt(StateInfo::ExtTableState { .. }) => true,
                ComplexOrFelt::Felt(StateInfo::PublicParam(_)) => true,
                ComplexOrFelt::Complex(children) => children.iter().all(|c| c.in_state()),
            }
        };

        let info = AirVarInfo {
            in_state,
            is_const: self.is_const,
            visibility: self.visibility,
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

impl<T> From<VarExpr<T>> for CompiledAirVar
where
    T: ProverType,
{
    fn from(v: VarExpr<T>) -> CompiledAirVar {
        // v is a constant
        if v.is_const {
            return CompiledAirVar::Const(
                T::r#type(),
                v.value.expect("Const must have a value").calc(),
            );
        }

        // v was written to the trace
        if let ComplexOrFelt::Felt(StateInfo::StateIndex(i, desc)) = v.complex_or_felt {
            return CompiledAirVar::State(State::get_cell_name(i, &desc));
        }

        // v was written to the trace of an external const table
        if let ComplexOrFelt::Felt(StateInfo::ExtTableState(name, args)) = v.complex_or_felt {
            return CompiledAirVar::ExternalState(name, args);
        }

        // v is a public param
        if let ComplexOrFelt::Felt(StateInfo::PublicParam(param)) = v.complex_or_felt {
            return CompiledAirVar::PublicParam(param.name());
        }

        // v is a field of another variable
        if let Some(parent) = v.parent {
            return parent.get_compiled_child();
        }

        // v is a standalone variable
        CompiledAirVar::Var(T::r#type(), v.name)
    }
}

#[derive(Clone, Debug)]
pub(super) struct ParentExpr {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) parent: Option<Box<ParentExpr>>,
    pub(super) index: Option<usize>,
    pub(super) child_name: String,
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
        if let Some(parent) = expr.parent {
            return parent.get_compiled_child();
        }
        CompiledAirVar::Var(expr.r#type, expr.name)
    }
}

// Each VarExpr is either a single felt, that can be written to the state
// or a complex expression that holds one or more expressions (children).
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
