use std::collections::HashSet;

use compiled_casm_air::compiled_structs::CompiledAirVar;
use compiled_casm_air::public_params::PublicParam;
use prover_types::cpu::ProverType;

use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;

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
    pub(super) intermediate_type: Option<IntermediateType>,
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
        intermediate_type: Option<IntermediateType>,
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
            intermediate_type,
        };
        var.create_children();
        var.update_children();
        var
    }

    pub fn new_const(value: T) -> Self {
        Self::new(value.calc(), Some(value), true, false, None)
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
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match &self.complex_or_felt {
            ComplexOrFelt::Felt(StateInfo::StateIndex(..)) => true,
            ComplexOrFelt::Felt(StateInfo::IsPolyOfState(b)) => *b,
            ComplexOrFelt::Felt(StateInfo::ExternalColumnStateIndex(..)) => true,
            ComplexOrFelt::Felt(StateInfo::PublicParam(_)) => true,
            ComplexOrFelt::Complex(children) => children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        self.is_const
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.intermediate_type.clone().map_or(vec![], |t| vec![t])
    }

    fn get_public_params(&self) -> HashSet<PublicParam> {
        let mut res = HashSet::new();
        if let ComplexOrFelt::Felt(StateInfo::PublicParam(ref p)) = self.complex_or_felt {
            res.insert(p.clone());
        }
        res
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
        if let ComplexOrFelt::Felt(StateInfo::ExternalColumnStateIndex(name, i)) = v.complex_or_felt
        {
            return CompiledAirVar::ExternalState(name, i);
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
