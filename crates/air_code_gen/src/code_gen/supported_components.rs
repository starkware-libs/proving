use std::path::PathBuf;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AutogenCodeType {
    WITNESS,
    AIR,
    CAIRO,
}

#[derive(Debug)]
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
        "verify_bitwise_xor_12".into(),
        "verify_instruction".into(),
    ]
}

/// Returns the list of components whose Cairo constraint evaluation code is manually written.
/// The CI ensures that constraint evaluation code generation works for all components not
/// listed here.
fn get_manual_cairo_constraints_components() -> Vec<String> {
    vec![
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "verify_bitwise_xor_12".into(),
        // TODO(AnatG): Make those changes in codegen or air compilation.
        "cube_252".into(),
        "poseidon_builtin".into(),
    ]
}

fn get_manual_witness_components() -> Vec<String> {
    vec![
        "blake_round".into(),
        "memory_address_to_id".into(),
        "memory_id_to_big".into(),
        "cube_252".into(),
        "partial_ec_mul".into(),
        "verify_instruction".into(),
        // Lookups to pre-processed tables
        "blake_round_sigma".into(),
        "pedersen_points_table".into(),
        "poseidon_round_keys".into(),
        "range_check_3_3_3_3_3".into(),
        "range_check_3_6_6_3".into(),
        "range_check_4_3".into(),
        "range_check_4_4_4_4".into(),
        "range_check_4_4".into(),
        "range_check_5_4".into(),
        "range_check_6".into(),
        "range_check_7_2_5".into(),
        "range_check_8".into(),
        "range_check_9_9".into(),
        "range_check_9_9_b".into(),
        "range_check_9_9_c".into(),
        "range_check_9_9_d".into(),
        "range_check_9_9_e".into(),
        "range_check_9_9_f".into(),
        "range_check_9_9_g".into(),
        "range_check_9_9_h".into(),
        "range_check_11".into(),
        "range_check_12".into(),
        "range_check_18".into(),
        "range_check_18_b".into(),
        "range_check_19".into(),
        "range_check_19_b".into(),
        "range_check_19_c".into(),
        "range_check_19_d".into(),
        "range_check_19_e".into(),
        "range_check_19_f".into(),
        "range_check_19_g".into(),
        "range_check_19_h".into(),
        "verify_bitwise_xor_4".into(),
        "verify_bitwise_xor_7".into(),
        "verify_bitwise_xor_8".into(),
        "verify_bitwise_xor_9".into(),
        "verify_bitwise_xor_12".into(),
    ]
}

/// Is code autogeneration supposed to work for the given file?
pub fn is_supported(job: &AutogenCodeFile) -> bool {
    match job.code_type {
        // We don't support autogeneration of witness-genenration code yet
        AutogenCodeType::WITNESS => !get_manual_witness_components().contains(&job.air_fn_name),

        AutogenCodeType::AIR => {
            !get_manual_rust_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CAIRO => {
            !get_manual_cairo_constraints_components().contains(&job.air_fn_name)
        }
    }
}
