use std::path::PathBuf;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AutogenCodeType {
    WITNESS,
    AIR,
    CAIRO,
    CIRCUIT,
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
        "verify_bitwise_xor_12".into(),
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

        AutogenCodeType::AIR => {
            !get_manual_rust_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CAIRO => {
            !get_manual_cairo_constraints_components().contains(&job.air_fn_name)
        }

        AutogenCodeType::CIRCUIT => [
            "add_ap_opcode".to_owned(),
            "assert_eq_opcode".to_owned(),
            "cond_range_check_2".to_owned(),
            "decode_instruction_15a61".to_owned(),
            "decode_instruction_7ebc4".to_owned(),
            "decode_instruction_d2a10".to_owned(),
            "decode_instruction_de75a".to_owned(),
            "decode_instruction_fe864".to_owned(),
            "decode_small_sign".to_owned(),
            "encode_offsets".to_owned(),
            "jump_opcode_rel_imm".to_owned(),
            "jnz_opcode_non_taken".to_owned(),
            "jnz_opcode_taken".to_owned(),
            "mem_verify".to_owned(),
            "mem_verify_equal".to_owned(),
            "range_check_11".to_owned(),
            "range_check_18".to_owned(),
            "range_check_29".to_owned(),
            "range_check_4_3".to_owned(),
            "range_check_7_2_5".to_owned(),
            "range_check_last_limb_bits_in_ms_limb_2".to_owned(),
            "read_id".to_owned(),
            "read_positive_known_id_num_bits_252".to_owned(),
            "read_positive_known_id_num_bits_29".to_owned(),
            "read_positive_num_bits_252".to_owned(),
            "read_positive_num_bits_29".to_owned(),
            "read_small".to_owned(),
            "ret_opcode".to_owned(),
            "verify_instruction".to_owned(),
        ]
        .contains(&job.air_fn_name),
    }
}
