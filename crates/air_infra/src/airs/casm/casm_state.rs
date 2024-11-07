use prover_types::cpu::{CasmState, ProverType};

use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::core::Felt;

#[derive(Clone, Debug, Default)]
pub struct CasmAddress {
    pub desc: Option<String>,
    pub value: FeltExpr,
}

impl CasmAddress {
    pub fn new(value: FeltExpr, desc: &str) -> Self {
        CasmAddress {
            desc: (!desc.is_empty()).then(|| desc.to_string()),
            value,
        }
    }
}

impl AsProverType<Felt> for CasmAddress {
    fn value(&self) -> Option<Felt> {
        self.value.value()
    }
}

impl From<CasmAddress> for AirVarImpl {
    fn from(address: CasmAddress) -> AirVarImpl {
        AirVarImpl::Expr(address.value.into())
    }
}

impl InternalAirVarInfo for CasmAddress {
    fn is_const(&self) -> bool {
        self.value.is_const()
    }

    fn in_state(&self) -> bool {
        self.value.in_state()
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.value.get_intermediate_types()
    }
}

impl InternalAirVarActions for CasmAddress {
    fn let_(&self, name: String, intermediate_type: IntermediateType) -> Self {
        CasmAddress {
            desc: self.desc.clone(),
            value: self.value.let_(name, intermediate_type.clone()),
        }
    }

    fn new(name: String, in_state: bool) -> Self {
        CasmAddress {
            desc: Some(name.clone()),
            value: FeltExpr::new(name, in_state),
        }
    }
}

impl AirVar for CasmAddress {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.value.as_felts_mut()
    }
}

#[derive(Clone, Debug)]
pub struct CasmStateVar {
    pub(super) name: Option<String>,
    pub pc: CasmAddress,
    pub ap: CasmAddress,
    pub fp: CasmAddress,
}

impl CasmStateVar {
    pub fn new(pc: FeltExpr, ap: FeltExpr, fp: FeltExpr) -> Self {
        CasmStateVar {
            name: None,
            pc: CasmAddress::new(pc, "pc"),
            ap: CasmAddress::new(ap, "ap"),
            fp: CasmAddress::new(fp, "fp"),
        }
    }
}

impl AsProverType<CasmState> for CasmStateVar {
    fn value(&self) -> Option<CasmState> {
        if let (Some(pc), Some(ap), Some(fp)) = (self.pc.value(), self.ap.value(), self.fp.value())
        {
            Some(CasmState { pc, ap, fp })
        } else {
            None
        }
    }
}

impl From<CasmStateVar> for AirVarImpl {
    fn from(state: CasmStateVar) -> AirVarImpl {
        AirVarImpl::Struct {
            name: state.name,
            r#type: CasmState::r#type(),
            fields: vec![
                ("pc".to_string(), state.pc.into()),
                ("ap".to_string(), state.ap.into()),
                ("fp".to_string(), state.fp.into()),
            ],
        }
    }
}

impl InternalAirVarInfo for CasmStateVar {
    fn is_const(&self) -> bool {
        self.pc.is_const() && self.ap.is_const() && self.fp.is_const()
    }

    fn in_state(&self) -> bool {
        self.pc.in_state() && self.ap.in_state() && self.fp.in_state()
    }

    fn get_intermediate_types(&self) -> Vec<IntermediateType> {
        self.pc
            .get_intermediate_types()
            .into_iter()
            .chain(self.ap.get_intermediate_types())
            .chain(self.fp.get_intermediate_types())
            .collect()
    }
}

impl InternalAirVarActions for CasmStateVar {
    fn let_(&self, name: String, intermediate_type: IntermediateType) -> Self {
        CasmStateVar {
            name: Some(name.clone()),
            pc: self
                .pc
                .let_(format!("{}.pc", name), intermediate_type.clone()),
            ap: self
                .ap
                .let_(format!("{}.ap", name), intermediate_type.clone()),
            fp: self.fp.let_(format!("{}.fp", name), intermediate_type),
        }
    }

    fn new(name: String, in_state: bool) -> Self {
        CasmStateVar {
            name: Some(name.clone()),
            pc: CasmAddress::new(FeltExpr::new(format!("{}.pc", name), in_state), "pc"),
            ap: CasmAddress::new(FeltExpr::new(format!("{}.ap", name), in_state), "ap"),
            fp: CasmAddress::new(FeltExpr::new(format!("{}.fp", name), in_state), "fp"),
        }
    }
}

impl AirVar for CasmStateVar {
    fn get_felt_descriptions(&self) -> Option<Vec<String>> {
        Some(vec!["pc".to_string(), "ap".to_string(), "fp".to_string()])
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.pc
            .as_felts_mut()
            .into_iter()
            .chain(self.ap.as_felts_mut())
            .chain(self.fp.as_felts_mut())
            .collect()
    }
}
