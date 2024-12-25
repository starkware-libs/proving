use std::fmt::Debug;
use std::marker::PhantomData;

use indexmap::IndexMap;
use prover_types::cpu::ProverType;

use super::expressions::felt_expr::*;
use super::variables::*;

pub trait StructVarTrait {
    fn new_from_name(name: String, in_state: bool) -> Self;
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

impl<F: AirVar, T: ProverType> InternalAirVarInfo for StructVar<F, T>
where
    Self: StructVarTrait,
{
    fn is_const(&self) -> bool {
        self.fields.iter().all(|(_, f)| f.is_const())
    }

    fn in_state(&self) -> bool {
        self.fields.iter().all(|(_, f)| f.in_state())
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.fields
            .iter()
            .flat_map(|(_, f)| f.get_intermediate_types())
            .collect()
    }

    fn prover_type(&self) -> String {
        <Self as StructVarTrait>::prover_type()
    }
}

impl<F: AirVar, T: ProverType> InternalAirVarActions for StructVar<F, T>
where
    Self: StructVarTrait,
{
    fn let_(&self, name: String, intermediate_type: IntermediateType) -> Self {
        Self {
            name: Some(name.clone()),
            fields: self
                .fields
                .iter()
                .map(|(n, f)| {
                    (
                        n.clone(),
                        f.let_(format!("{}.{}", name, n), intermediate_type.clone()),
                    )
                })
                .collect(),
            r#type: PhantomData,
        }
    }

    fn new(name: String, in_state: bool) -> Self {
        Self::new_from_name(name, in_state)
    }
}

impl<F: AirVar, T: ProverType> AirVar for StructVar<F, T>
where
    Self: StructVarTrait,
{
    fn get_felt_descriptions(&self) -> Option<Vec<String>> {
        Some(
            self.fields
                .iter()
                .flat_map(|(n, f)| {
                    if let Some(descs) = f.get_felt_descriptions() {
                        descs.iter().map(|d| format!("{}_{}", n, d)).collect()
                    } else {
                        let n_felts = f.as_felts().len();
                        vec![n.clone(); n_felts]
                    }
                })
                .collect(),
        )
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.fields
            .iter_mut()
            .flat_map(|(_, f)| f.as_felts_mut())
            .collect()
    }
}

// VarWrapper wraps an air var with an optional description.
// It is compiled by compiling its field. See for example CasmAddress.
#[derive(Clone, Debug)]
pub struct VarWrapper<V: AirVar> {
    pub var: V,
    pub desc: Option<String>,
}

impl<V: AirVar> VarWrapper<V> {
    pub fn new(var: V, desc: &str) -> Self {
        Self {
            var,
            desc: (!desc.is_empty()).then(|| desc.to_string()),
        }
    }
}

impl<V: AirVar, T: ProverType> AsProverType<T> for VarWrapper<V>
where
    V: AsProverType<T>,
{
    fn value(&self) -> Option<T> {
        self.var.value()
    }
}

impl<V: AirVar> From<VarWrapper<V>> for AirVarImpl {
    fn from(v: VarWrapper<V>) -> AirVarImpl {
        v.var.into()
    }
}

impl<V: AirVar> InternalAirVarInfo for VarWrapper<V> {
    fn is_const(&self) -> bool {
        self.var.is_const()
    }

    fn in_state(&self) -> bool {
        self.var.in_state()
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.var.get_intermediate_types()
    }

    fn prover_type(&self) -> String {
        self.var.prover_type()
    }
}

impl<V: AirVar> InternalAirVarActions for VarWrapper<V> {
    fn let_(&self, name: String, intermediate_type: IntermediateType) -> Self {
        Self {
            var: self.var.let_(name, intermediate_type),
            desc: self.desc.clone(),
        }
    }

    fn new(name: String, in_state: bool) -> Self {
        Self {
            var: V::new(name.clone(), in_state),
            desc: Some(name),
        }
    }
}

impl<V: AirVar> AirVar for VarWrapper<V> {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.var.as_felts_mut()
    }
}
