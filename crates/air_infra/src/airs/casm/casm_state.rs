use std::marker::PhantomData;

use indexmap::IndexMap;
use stwo_cairo_common::prover_types::cpu::{CasmState, ProverType};

use crate::core::expressions::felt_expr::*;
use crate::core::struct_var::*;
use crate::core::variables::*;

pub type CasmAddress = VarWrapper<FeltExpr>;
pub type CasmStateVar = StructVar<CasmAddress, CasmState>;

impl StructVarTrait for CasmStateVar {
    fn new_from_name(name: String, in_state: bool) -> Self {
        StructVar {
            name: Some(name.clone()),
            fields: IndexMap::from([
                (
                    "pc".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{}.pc", name), in_state), "pc"),
                ),
                (
                    "ap".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{}.ap", name), in_state), "ap"),
                ),
                (
                    "fp".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{}.fp", name), in_state), "fp"),
                ),
            ]),
            r#type: PhantomData,
        }
    }

    fn prover_type() -> String {
        CasmState::r#type()
    }
}

impl CasmStateVar {
    pub fn new(pc: FeltExpr, ap: FeltExpr, fp: FeltExpr) -> Self {
        StructVar {
            name: None,
            fields: IndexMap::from([
                ("pc".to_string(), CasmAddress::new(pc, "pc")),
                ("ap".to_string(), CasmAddress::new(ap, "ap")),
                ("fp".to_string(), CasmAddress::new(fp, "fp")),
            ]),
            r#type: PhantomData,
        }
    }

    pub fn pc(&self) -> CasmAddress {
        self.fields.get("pc").expect("CasmState has pc").clone()
    }

    pub fn ap(&self) -> CasmAddress {
        self.fields.get("ap").expect("CasmState has ap").clone()
    }

    pub fn fp(&self) -> CasmAddress {
        self.fields.get("fp").expect("CasmState has fp").clone()
    }
}

impl AsProverType<CasmState> for CasmStateVar {
    fn value(&self) -> Option<CasmState> {
        if let (Some(pc), Some(ap), Some(fp)) =
            (self.pc().value(), self.ap().value(), self.fp().value())
        {
            Some(CasmState { pc, ap, fp })
        } else {
            None
        }
    }
}
