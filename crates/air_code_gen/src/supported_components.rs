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
    // `eq` is manual too, but stwo-air-infra has no compiled JSON for it, so it is not listed here.
    vec!["memory_address_to_id".into(), "memory_id_to_big".into(), "verify_bitwise_xor_12".into()]
}

/// Returns the list of components whose Cairo constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
///
/// The Cairo jobs are generated from `outputs/compiled_circuit_air`, so only circuit components
/// belong here.
pub(crate) fn get_manual_cairo_constraints_components() -> Vec<String> {
    // `eq` is manual too, but stwo-air-infra has no compiled JSON for it, so it is not listed here.
    vec!["verify_bitwise_xor_12".into()]
}

/// Returns the list of components whose circuit constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
fn get_manual_circuit_constraints_components() -> Vec<String> {
    // `eq` is manual too, but stwo-air-infra has no compiled JSON for it, so it is not listed here.
    vec![
        // CASM components
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        // Gates components
        "verify_bitwise_xor_12".into(),
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
