use std::path::PathBuf;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AirAutogenConfig {
    /// Additional traits that should be #[derive]-ed on the Claim and InteractionClaim structs
    pub additional_claim_traits: &'static [&'static str],
    /// Absolute path to the prelude module (e.g. "crate::components::prelude")
    pub prelude_import_path: &'static str,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AutogenCodeType {
    WITNESS,
    AIR(AirAutogenConfig),
    CAIRO,
    CIRCUIT,
}

#[derive(Clone, Debug)]
pub struct AutogenCodeFile {
    pub air_fn_name: String,
    /// Path of the source JSON
    pub source_path: PathBuf,
    /// The directory where to place the result. The filename and subdirectory (e.g.
    /// "subroutines/") are determined from the AirFn itself.
    pub dest_dir: PathBuf,
    pub code_type: AutogenCodeType,
}

/// Returns the list of components whose Rust constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
pub fn get_manual_rust_constraints_components() -> Vec<String> {
    vec![
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "verify_bitwise_xor_12".into(),
        "circuit_blake_round".into(),
        "qm_31_into_u_32".into(),
        "blake_gate".into(),
    ]
}

/// Returns the list of components whose Cairo constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
fn get_manual_cairo_constraints_components() -> Vec<String> {
    vec![
        // CASM components
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "cube_252".into(),
        "poseidon_aggregator".into(),
        "verify_bitwise_xor_12".into(),
        // We do not support pedersen narrow windows in Cairo.
        // In particular, the ppt with the columns of pedersen_points_table_window_bits_9 doesn't
        // exist, so the sample evaluation test of this component fails.
        "pedersen_points_table_window_bits_9".into(),
        // Gates components
        "qm_31_ops".into(),
        // The circuit version requires applying the enabler to the reading of the message,
        "circuit_blake_round".into(),
        "verify_bitwise_xor_12".into(),
        "qm_31_into_u_32".into(),
        "blake_gate".into(),
    ]
}

fn get_manual_circuit_constraints_components() -> Vec<String> {
    vec![
        // CASM components
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        // Gates components
        "qm_31_ops".into(),
        // The circuit version requires applying the enabler to the reading of the message,
        "circuit_blake_round".into(),
        "verify_bitwise_xor_12".into(),
        "qm_31_into_u_32".into(),
        "blake_gate".into(),
    ]
}

fn get_manual_witness_components() -> Vec<String> {
    vec![
        "blake_round".into(),
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "memory_id_to_small".into(),
        "cube_252".into(),
        "partial_ec_mul_window_bits_18".into(),
        "partial_ec_mul_window_bits_9".into(),
        "verify_bitwise_xor_12".into(),
    ]
}

/// Is code autogeneration supposed to work for the given file?
pub fn is_supported(job: &AutogenCodeFile) -> bool {
    match job.code_type {
        AutogenCodeType::WITNESS => !get_manual_witness_components().contains(&job.air_fn_name),

        AutogenCodeType::AIR(_) => {
            !get_manual_rust_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CAIRO => {
            !get_manual_cairo_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CIRCUIT => {
            !get_manual_circuit_constraints_components().contains(&job.air_fn_name)
        }
    }
}
