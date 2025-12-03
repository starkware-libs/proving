use std::path::PathBuf;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AutogenCodeType {
    WITNESS,
    AIR,
    CAIRO,
}

#[derive(Clone, Debug)]
pub struct AutogenCodeFile {
    pub air_fn_name: String,
    /// Path of the source JSON
    pub source_path: PathBuf,
    pub code_type: AutogenCodeType,
}

/// Returns the list of components whose Rust constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
pub fn get_manual_rust_constraints_components() -> Vec<String> {
    vec![
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "memory_id_to_small".into(),
    ]
}

/// Returns the list of components whose Cairo constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
fn get_manual_cairo_constraints_components() -> Vec<String> {
    vec![
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "memory_id_to_small".into(),
        // TODO(AnatG): Make those changes in codegen or air compilation.
        "cube_252".into(),
        "poseidon_aggregator".into(),
    ]
}

fn get_manual_witness_components() -> Vec<String> {
    vec![
        "blake_round".into(),
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "memory_id_to_small".into(),
        "cube_252".into(),
        "partial_ec_mul_bits_per_window_18".into(),
        "partial_ec_mul_bits_per_window_9".into(),
    ]
}

/// Is code autogeneration supposed to work for the given file?
pub fn is_supported(job: &AutogenCodeFile) -> bool {
    match job.code_type {
        AutogenCodeType::WITNESS => !get_manual_witness_components().contains(&job.air_fn_name),

        AutogenCodeType::AIR => {
            !get_manual_rust_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CAIRO => {
            !get_manual_cairo_constraints_components().contains(&job.air_fn_name)
        }
    }
}
