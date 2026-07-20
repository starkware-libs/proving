use std::marker::PhantomData;

use indexmap::IndexMap;
use stwo_cairo_common::prover_types::cpu::{CasmState, ProverType};

use crate::const_expr;
use crate::core::expressions::felt_expr::*;
use crate::core::struct_var::*;
use crate::core::variables::*;

// The <extra_info> saved for <CasmAddress> is the description, in name format, of what is written
// in that address.
pub type CasmAddress = VarWrapper<FeltExpr, String>;
pub type CasmStateVar = StructVar<CasmAddress, CasmState>;

impl CasmAddress {
    pub fn new(expr: FeltExpr, extra_info: &str) -> Self {
        Self { var: expr, extra_info: (!extra_info.is_empty()).then(|| extra_info.to_string()) }
    }
}

impl Default for CasmAddress {
    fn default() -> Self {
        Self { var: const_expr!(0), extra_info: None }
    }
}

impl StructVarTrait for CasmStateVar {
    fn new_from_name(name: String, deg_in_state: Option<usize>) -> Self {
        Self {
            name: Some(name.clone()),
            fields: IndexMap::from([
                (
                    "pc".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{name}.pc"), deg_in_state), "pc"),
                ),
                (
                    "ap".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{name}.ap"), deg_in_state), "ap"),
                ),
                (
                    "fp".to_string(),
                    CasmAddress::new(FeltExpr::new(format!("{name}.fp"), deg_in_state), "fp"),
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
        Self {
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
