use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::AirBuilder;
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::variables::ExtTable;
use air_infra::{const_bool_expr, const_expr};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

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
        vec!["qm31_ops_mults".to_string()]
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
        vec!["qm31_ops_in0_address".to_string()]
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
        vec!["qm31_ops_in1_address".to_string()]
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
        vec!["qm31_ops_out_address".to_string()]
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
        vec!["qm31_ops_add_flag".to_string()]
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
        vec!["qm31_ops_sub_flag".to_string()]
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
        vec!["qm31_ops_mul_flag".to_string()]
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
        vec!["qm31_ops_pointwise_mul_flag".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        const_bool_expr!(false)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TripleXorInputAddr0 {}
impl ExtTable for TripleXorInputAddr0 {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["triple_xor_input_addr_0".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct TripleXorInputAddr1 {}
impl ExtTable for TripleXorInputAddr1 {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["triple_xor_input_addr_1".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct TripleXorInputAddr2 {}
impl ExtTable for TripleXorInputAddr2 {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["triple_xor_input_addr_2".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct TripleXorOutputAddr {}
impl ExtTable for TripleXorOutputAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["triple_xor_output_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct TripleXorMultiplicity {}
impl ExtTable for TripleXorMultiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["triple_xor_multiplicity".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateMultiplicity {}
impl ExtTable for BlakeGGateMultiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_multiplicity".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrA {}
impl ExtTable for BlakeGGateInputAddrA {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_a".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrB {}
impl ExtTable for BlakeGGateInputAddrB {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_b".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrC {}
impl ExtTable for BlakeGGateInputAddrC {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_c".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrD {}
impl ExtTable for BlakeGGateInputAddrD {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_d".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrF0 {}
impl ExtTable for BlakeGGateInputAddrF0 {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_f0".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateInputAddrF1 {}
impl ExtTable for BlakeGGateInputAddrF1 {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_input_addr_f1".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateOutputAddrA {}
impl ExtTable for BlakeGGateOutputAddrA {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_output_addr_a".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateOutputAddrB {}
impl ExtTable for BlakeGGateOutputAddrB {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_output_addr_b".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateOutputAddrC {}
impl ExtTable for BlakeGGateOutputAddrC {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_output_addr_c".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlakeGGateOutputAddrD {}
impl ExtTable for BlakeGGateOutputAddrD {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["blake_g_gate_output_addr_d".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct M31ToU32InputAddr {}
impl ExtTable for M31ToU32InputAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["m31_to_u32_input_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct M31ToU32OutputAddr {}
impl ExtTable for M31ToU32OutputAddr {
    type T = CasmAddress;

    fn column_ids() -> Vec<String> {
        vec!["m31_to_u32_output_addr".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
pub struct M31ToU32Multiplicity {}
impl ExtTable for M31ToU32Multiplicity {
    type T = FeltExpr;

    fn column_ids() -> Vec<String> {
        vec!["m31_to_u32_multiplicity".to_string()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}
