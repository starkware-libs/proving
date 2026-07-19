use std::fmt::Debug;
use std::marker::PhantomData;

use indexmap::IndexMap;
use stwo_cairo_common::prover_types::cpu::ProverType;

use super::expressions::felt_expr::*;
use super::variables::*;

pub trait StructVarTrait {
    fn new_from_name(name: String, deg_in_state: Option<usize>) -> Self;
    fn prover_type() -> String;
}

// StructVar is a generic struct that holds several fields of the same type F.
// It is compiled into a struct with type T.
// See for example CasmStateVar.
#[derive(Clone, Debug)]
pub struct StructVar<F: AirVar, T: ProverType> {
    pub name: Option<String>,
    pub fields: IndexMap<String, F>,
    pub r#type: PhantomData<T>,
}

impl<F: AirVar, T: ProverType> From<StructVar<F, T>> for AirVarImpl {
    fn from(v: StructVar<F, T>) -> AirVarImpl {
        AirVarImpl::Struct {
            name: v.name,
            r#type: T::r#type(),
            fields: v.fields.into_iter().map(|(n, f)| (n, f.into())).collect(),
        }
    }
}

impl<F: AirVar, T: ProverType> AirVar for StructVar<F, T>
where
    Self: StructVarTrait,
{
    fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
        let interm = Intermediate::new_for_deduction(&name, self);
        let res = Self {
            name: Some(name.clone()),
            fields: self
                .fields
                .iter()
                .map(|(n, f)| (n.clone(), f.let_for_deduction(format!("{name}.{n}")).0))
                .collect(),
            r#type: PhantomData,
        };
        (res, interm)
    }

    fn new(name: String, deg_in_state: Option<usize>) -> Self {
        Self::new_from_name(name, deg_in_state)
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.fields.iter_mut().flat_map(|(_, f)| f.as_felts_mut()).collect()
    }
}

// VarWrapper wraps an air var with an optional description.
// It is compiled by compiling its field. See for example CasmAddress.
#[derive(Clone, Debug)]
pub struct VarWrapper<V: AirVar, D: Clone + Debug> {
    pub var: V,
    pub extra_info: Option<D>,
}

impl<V: AirVar, T: ProverType, D: Clone + Debug> AsProverType<T> for VarWrapper<V, D>
where
    V: AsProverType<T>,
{
    fn value(&self) -> Option<T> {
        self.var.value()
    }
}

impl<V: AirVar, D: Clone + Debug> From<VarWrapper<V, D>> for AirVarImpl {
    fn from(v: VarWrapper<V, D>) -> AirVarImpl {
        v.var.into()
    }
}

impl<V: AirVar, D: Clone + Debug> AirVar for VarWrapper<V, D> {
    fn let_for_deduction(&self, name: String) -> (Self, Intermediate) {
        let interm = Intermediate::new_for_deduction(&name, self);
        let res =
            Self { var: self.var.let_for_deduction(name).0, extra_info: self.extra_info.clone() };
        (res, interm)
    }

    fn new(name: String, deg_in_state: Option<usize>) -> Self {
        Self { var: V::new(name.clone(), deg_in_state), extra_info: None }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.var.as_felts_mut()
    }
}
