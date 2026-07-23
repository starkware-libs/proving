use std::collections::HashSet;

use air_compile::compiled_structs::CompiledAirVar;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::super::air_body::*;
use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use crate::core::Felt;

pub trait VarExprUpdate {
    fn create_complex_or_felt(&mut self, is_const: bool, deg_in_state: Option<usize>);
    fn update_children(&mut self);
}

#[derive(Clone, Debug)]
pub struct VarExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    pub(super) value: Option<T>,
    pub(super) parent: Option<ParentExpr>,
    pub(super) complex_or_felt: ComplexOrFelt,
    pub(super) is_deduction_intermediate: bool,
    // Every variable can be <in_deductions>, but only felts can be <in_constraints>.
    pub(super) visibility: Visibility,
}

impl<T> VarExpr<T>
where
    T: ProverType,
{
    pub fn new(name: String, value: Option<T>, is_const: bool, deg_in_state: Option<usize>) -> Self
    where
        Self: VarExprUpdate,
    {
        if is_const {
            assert!(value.is_some());
        }

        let mut var = VarExpr {
            name,
            value,
            parent: None,
            // This is updated when calling <create_complex_or_felt>.
            complex_or_felt: ComplexOrFelt::default(),
            is_deduction_intermediate: false,
            visibility: Visibility {
                in_deductions: true,
                in_constraints: is_const && T::r#type() == Felt::r#type(),
            },
        };
        var.create_complex_or_felt(is_const, deg_in_state);
        var.update_children();
        var
    }

    pub fn new_const(value: T) -> Self
    where
        Self: VarExprUpdate,
    {
        Self::new(value.calc(), Some(value), true, Some(0))
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
        };

        self.parent = Some(parent);
        self.update_children();
    }

    pub(super) fn get_felt_children(&mut self) -> Vec<&mut FeltExpr> {
        self.complex_or_felt
            .as_complex_mut()
            .iter_mut()
            .map(|c| c.as_felt_mut())
            .collect::<Vec<_>>()
    }

    pub(super) fn as_felt_mut(&mut self) -> &mut FeltExpr {
        assert!(self.complex_or_felt.as_complex().len() == 1, "Expected single felt");
        self.complex_or_felt.as_complex_mut().get_mut(0).expect("Invalid index").as_felt_mut()
    }

    pub(super) fn as_felt(&self) -> FeltExpr {
        assert!(self.complex_or_felt.as_complex().len() == 1, "Expected single felt");
        self.complex_or_felt.as_complex().first().expect("Invalid index").as_felt()
    }

    pub(super) fn get_felt_mut(&mut self, index: usize) -> &mut FeltExpr {
        self.complex_or_felt.as_complex_mut().get_mut(index).expect("Invalid index").as_felt_mut()
    }

    pub(super) fn get_felt(&self, index: usize) -> FeltExpr {
        self.complex_or_felt.as_complex().get(index).expect("Invalid index").as_felt()
    }

    fn is_const(&self) -> bool {
        match &self.complex_or_felt {
            ComplexOrFelt::Felt(FeltInfo { is_const, .. }) => *is_const,
            ComplexOrFelt::Complex(children) => children.iter().all(|c| c.is_const()),
        }
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

impl<T> AirVarImplInfo for VarExpr<T>
where
    T: ProverType,
{
    fn var_expr_infos(&self) -> HashSet<VarExprInfo> {
        let info = VarExprInfo {
            is_const: self.is_const(),
            visibility: self.visibility,
            public_param: if let ComplexOrFelt::Felt(FeltInfo {
                value_info: ValueInfo::PublicParam(ref p),
                constraint_intermediate: _,
                is_const: _,
            }) = self.complex_or_felt
            {
                Some(p.clone())
            } else {
                None
            },
            external_state: if let ComplexOrFelt::Felt(FeltInfo {
                value_info: ValueInfo::ExternalState(ext_state),
                constraint_intermediate: _,
                is_const: _,
            }) = self.complex_or_felt.clone()
            {
                Some(ext_state)
            } else {
                None
            },
        };
        HashSet::from([info])
    }

    fn deg_in_state(&self) -> Option<usize> {
        if T::r#type() != Felt::r#type() {
            panic!("Only felt variables can have a degree in state");
        }

        if self.is_const() {
            Some(0)
        } else {
            match &self.complex_or_felt.as_felt_info().value_info {
                ValueInfo::StateIndex(..) => Some(1),
                ValueInfo::DegPolyOfState(deg) => *deg,
                ValueInfo::ExternalState { .. } => Some(1),
                ValueInfo::PublicParam(_) => Some(0),
                ValueInfo::Enabler => Some(1),
                ValueInfo::Multiplicity(_) => Some(1),
            }
        }
    }

    fn prover_type(&self) -> String {
        T::r#type()
    }

    fn compile(self, compile_for: CompileFor) -> CompiledAirVar {
        // self is a constant
        if self.is_const() {
            return CompiledAirVar::Const(
                T::r#type(),
                self.value.expect("Const must have a value").calc(),
            );
        }

        // self was written to the trace
        if let ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::StateIndex(i, desc),
            constraint_intermediate: _,
            is_const: _,
        }) = self.complex_or_felt
        {
            return CompiledAirVar::State(State::get_cell_name(i, &desc));
        }

        // self was written to the trace of an external const table
        if let ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::ExternalState(ext_state),
            constraint_intermediate: _,
            is_const: _,
        }) = self.complex_or_felt
        {
            return CompiledAirVar::ExternalState(ext_state);
        }

        // self is a public param
        if let ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::PublicParam(param),
            constraint_intermediate: _,
            is_const: _,
        }) = self.complex_or_felt
        {
            return CompiledAirVar::PublicParam(param.name());
        }

        // self is the enabler value
        if let ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::Enabler,
            constraint_intermediate: _,
            is_const: _,
        }) = self.complex_or_felt
        {
            return CompiledAirVar::Enabler;
        }

        // self is a multiplicity value
        if let ComplexOrFelt::Felt(FeltInfo {
            value_info: ValueInfo::Multiplicity(idx),
            constraint_intermediate: _,
            is_const: _,
        }) = self.complex_or_felt
        {
            return CompiledAirVar::Multiplicity(idx);
        }

        if compile_for == CompileFor::Deductions {
            // self is an intermediate visible in deductions
            if self.is_deduction_intermediate {
                return CompiledAirVar::Var(T::r#type(), self.name);
            }
        } else {
            // <compile_for> == CompileFor::Constraints
            // self is an intermediate visible in constraints
            if let ComplexOrFelt::Felt(FeltInfo {
                value_info: _,
                constraint_intermediate: Some(name),
                is_const: _,
            }) = self.complex_or_felt
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

// Each VarExpr is either a single felt, that can be written to the state and an intermediate in
// constraints, or a complex expression that holds one or more expressions (children).
#[derive(Clone, Debug)]
pub(super) enum ComplexOrFelt {
    Complex(Vec<ExprImpl>),
    Felt(FeltInfo),
}

impl Default for ComplexOrFelt {
    fn default() -> Self {
        ComplexOrFelt::Complex(vec![])
    }
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

    pub(super) fn as_felt_info_mut(&mut self) -> &mut FeltInfo {
        if let ComplexOrFelt::Felt(felt) = self {
            return felt;
        }
        panic!("Expected felt expression");
    }

    pub(super) fn as_felt_info(&self) -> &FeltInfo {
        if let ComplexOrFelt::Felt(felt) = self {
            return felt;
        }
        panic!("Expected felt expression");
    }
}
