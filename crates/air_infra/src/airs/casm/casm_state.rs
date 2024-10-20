use compiled_casm_air::prover_types::{CasmState, ProverType};

use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

#[derive(Clone, Debug)]
pub struct CasmStateVar {
    pub(super) name: Option<String>,
    pub pc: FeltExpr,
    pub ap: FeltExpr,
    pub fp: FeltExpr,
}

impl CasmStateVar {
    pub fn new(pc: FeltExpr, ap: FeltExpr, fp: FeltExpr) -> Self {
        CasmStateVar {
            name: None,
            pc,
            ap,
            fp,
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

    fn new(name: String) -> Self {
        CasmStateVar {
            name: Some(name.clone()),
            pc: FeltExpr::new(format!("{}.pc", name)),
            ap: FeltExpr::new(format!("{}.ap", name)),
            fp: FeltExpr::new(format!("{}.fp", name)),
        }
    }
}

impl AirVar for CasmStateVar {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        self.pc
            .as_felts_mut()
            .into_iter()
            .chain(self.ap.as_felts_mut())
            .chain(self.fp.as_felts_mut())
            .collect()
    }
}
