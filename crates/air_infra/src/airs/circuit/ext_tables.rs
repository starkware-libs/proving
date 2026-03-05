use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

use crate::airs::casm::casm_state::CasmAddress;
use crate::core::air_fn::AirBuilder;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::ExtTable;
use crate::{const_bool_expr, const_expr};

#[derive(Debug, Default, Clone)]
pub struct BlakeGateEnabler {}
impl ExtTable for BlakeGateEnabler {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["compress_enabler".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Message0Addr {}
impl ExtTable for Message0Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["message0_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Message1Addr {}
impl ExtTable for Message1Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["message1_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Message2Addr {}
impl ExtTable for Message2Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["message2_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Message3Addr {}
impl ExtTable for Message3Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["message3_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct StateBeforeAddr {}
impl ExtTable for StateBeforeAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["state_before_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct StateAfterAddr {}
impl ExtTable for StateAfterAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["state_after_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct FinalStateAddr {}
impl ExtTable for FinalStateAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["final_state_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeOutput0Multiplicity {}
impl ExtTable for BlakeOutput0Multiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["blake_output0_mults".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeOutput0Addr {}
impl ExtTable for BlakeOutput0Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_output0_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeOutput1Multiplicity {}
impl ExtTable for BlakeOutput1Multiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["blake_output1_mults".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeOutput1Addr {}
impl ExtTable for BlakeOutput1Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_output1_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct QM31OpsMultiplicity {}
impl ExtTable for QM31OpsMultiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["qm31_ops_multiplicity".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Op0Addr {}
impl ExtTable for Op0Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["op0_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Op1Addr {}
impl ExtTable for Op1Addr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["op1_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct DstAddr {}
impl ExtTable for DstAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["dst_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeT {}
impl ExtTable for BlakeT {
    type T = [FeltExpr; 2];

    fn column_ids() -> Vec<String> {
        vec!["t0".to_string(), "t1".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        [const_expr!(64), const_expr!(0)]
    }
}

#[derive(Debug, Default, Clone)]
pub struct FinalizeFlag {}
impl ExtTable for FinalizeFlag {
    type T = BoolExpr;

    fn column_ids() -> Vec<String> {
        vec!["finalize_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(false)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddFlag {}
impl ExtTable for AddFlag {
    type T = BoolExpr;

    fn column_ids() -> Vec<String> {
        vec!["add_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(false)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubFlag {}
impl ExtTable for SubFlag {
    type T = BoolExpr;

    fn column_ids() -> Vec<String> {
        vec!["sub_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(false)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MulFlag {}
impl ExtTable for MulFlag {
    type T = BoolExpr;

    fn column_ids() -> Vec<String> {
        vec!["mul_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(true)
    }
}

#[derive(Debug, Default, Clone)]
pub struct PointwiseMulFlag {}
impl ExtTable for PointwiseMulFlag {
    type T = BoolExpr;

    fn column_ids() -> Vec<String> {
        vec!["pointwise_mul_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(false)
    }
}
