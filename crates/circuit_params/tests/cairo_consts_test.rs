//! Keeps the Cairo circuit verifier's hardcoded constants in step with the production circuit
//! registry: derives its shared target sizes in memory from the committed definition
//! (`circuit_registry_definitions/production/definition.json`), renders the generated sections of
//! `multiverifier_consts.cairo` and `preprocessed_columns.cairo` from them, and asserts they match
//! the committed files. Run with `FIX=1` to rewrite the sections.
//!
//! The layout order comes from `layout_from_component_sizes` — the same function the recursive
//! tree derives the layout from — so the verifier's column indices cannot drift from the prover's
//! commitment order.

use std::fmt::Write as _;
use std::path::PathBuf;

use circuit_common::finalize::ComponentSizes;
use circuit_common::preprocessed::layout_from_component_sizes;
use circuit_params::RegistryDefinition;
use stwo::core::fri::FriConfig;

const BEGIN_MARKER: &str =
    "// === BEGIN GENERATED (see cairo_consts_test.rs; running it with FIX=1 regenerates) ===";
const END_MARKER: &str = "// === END GENERATED ===";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn circuit_air_src() -> PathBuf {
    repo_root().join("stwo_cairo_verifier/crates/circuit_air/src")
}

/// The shared layout in commitment order, as `(id, log_size)` pairs.
fn layout(target_sizes: &ComponentSizes) -> Vec<(String, u32)> {
    layout_from_component_sizes(target_sizes)
        .iter()
        .map(|(id, log_size)| (id.id.clone(), *log_size))
        .collect()
}

/// The Cairo `PerComponent` field a preprocessed column id belongs to. The fixed lookup tables
/// belong to their verifying components, whose Cairo names are legacy (`range_check_16` owns
/// `seq_16`; `verify_bitwise_xor_12` verifies 12-bit xor via the 10-bit table).
fn cairo_component(id: &str) -> &'static str {
    match id {
        _ if id.starts_with("eq_") => "eq",
        _ if id.starts_with("qm31_ops_") => "qm31_ops",
        _ if id.starts_with("triple_xor_") => "triple_xor",
        _ if id.starts_with("m31_to_u32_") => "m_31_to_u_32",
        _ if id.starts_with("blake_g_gate_") => "blake_g_gate",
        "seq_16" => "range_check_16",
        _ if id.starts_with("bitwise_xor_4_") => "verify_bitwise_xor_4",
        _ if id.starts_with("bitwise_xor_7_") => "verify_bitwise_xor_7",
        _ if id.starts_with("bitwise_xor_8_") => "verify_bitwise_xor_8",
        _ if id.starts_with("bitwise_xor_9_") => "verify_bitwise_xor_9",
        _ if id.starts_with("bitwise_xor_10_") => "verify_bitwise_xor_12",
        _ => panic!("unknown preprocessed column id {id:?}"),
    }
}

/// The Cairo `*_IDX` constant name of a preprocessed column id. The component code
/// references some columns under legacy names, aliased here (`qm31_ops_*` without the prefix and
/// with `in0/in1/out` as `OP_0/OP_1/DST`).
fn idx_const_name(id: &str) -> String {
    let renamed = match id {
        "qm31_ops_add_flag" => "add_flag",
        "qm31_ops_sub_flag" => "sub_flag",
        "qm31_ops_mul_flag" => "mul_flag",
        "qm31_ops_pointwise_mul_flag" => "pointwise_mul_flag",
        "qm31_ops_in0_address" => "op_0_addr",
        "qm31_ops_in1_address" => "op_1_addr",
        "qm31_ops_out_address" => "dst_addr",
        "qm31_ops_mults" => "qm_31_ops_multiplicity",
        "blake_g_gate_input_addr_f0" => "blake_g_gate_input_addr_f_0",
        "blake_g_gate_input_addr_f1" => "blake_g_gate_input_addr_f_1",
        _ if id.starts_with("m31_to_u32_") => {
            return format!("M_31_TO_U_32_{}_IDX", id["m31_to_u32_".len()..].to_uppercase());
        }
        _ => id,
    };
    format!("{}_IDX", renamed.to_uppercase())
}

/// Renders the generated section of `multiverifier_consts.cairo`: the pinned PCS config and the
/// component / preprocessed-column log sizes.
fn render_multiverifier_consts(fri_config: FriConfig, target_sizes: &ComponentSizes) -> String {
    let layout = layout(target_sizes);
    let log_of = |id: &str| {
        layout.iter().find(|(col, _)| col == id).unwrap_or_else(|| panic!("missing {id}")).1
    };
    let FriConfig {
        pow_bits,
        log_blowup_factor,
        log_last_layer_degree_bound,
        n_queries,
        fold_step,
    } = fri_config;

    let mut out = String::new();
    let w = &mut out;
    writeln!(w, "{BEGIN_MARKER}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// Expected PCS config of the multiverifier circuit's proof.").unwrap();
    writeln!(w, "///").unwrap();
    writeln!(w, "/// Pinned to the production registry's proof config, so the verifier accepts")
        .unwrap();
    writeln!(w, "/// only proofs produced with that canonical configuration. This pins").unwrap();
    writeln!(w, "/// every FRI security parameter (a weaker config — fewer queries, smaller")
        .unwrap();
    writeln!(w, "/// blowup, or less proof-of-work — is rejected, independently of stwo's")
        .unwrap();
    writeln!(w, "/// `security_bits >= SECURITY_BITS` floor).").unwrap();
    writeln!(
        w,
        "/// Note `pow_bits + log_blowup_factor * n_queries = {pow_bits} + {log_blowup_factor} * \
         {n_queries} = {} = SECURITY_BITS`.",
        pow_bits as usize + log_blowup_factor as usize * n_queries
    )
    .unwrap();
    writeln!(w, "pub fn circuit_pcs_config() -> PcsConfig {{").unwrap();
    writeln!(w, "    PcsConfig {{").unwrap();
    writeln!(w, "        fri_config: FriConfig {{").unwrap();
    writeln!(w, "            pow_bits: {pow_bits},").unwrap();
    writeln!(w, "            log_blowup_factor: {log_blowup_factor},").unwrap();
    writeln!(w, "            log_last_layer_degree_bound: {log_last_layer_degree_bound},").unwrap();
    writeln!(w, "            n_queries: {n_queries},").unwrap();
    writeln!(w, "            fold_step: {fold_step},").unwrap();
    writeln!(w, "        }},").unwrap();
    writeln!(w, "    }}").unwrap();
    writeln!(w, "}}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// Each component's log size.").unwrap();
    writeln!(w, "pub const COMPONENT_LOG_SIZES: PerComponent<u32> = PerComponent {{").unwrap();
    writeln!(w, "    eq: {},", log_of("eq_in0_address")).unwrap();
    writeln!(w, "    qm31_ops: {},", log_of("qm31_ops_add_flag")).unwrap();
    writeln!(w, "    triple_xor: {},", log_of("triple_xor_input_addr_0")).unwrap();
    writeln!(w, "    m_31_to_u_32: {},", log_of("m31_to_u32_input_addr")).unwrap();
    writeln!(w, "    blake_g_gate: {},", log_of("blake_g_gate_input_addr_a")).unwrap();
    writeln!(w, "    verify_bitwise_xor_8: {},", log_of("bitwise_xor_8_0")).unwrap();
    writeln!(w, "    verify_bitwise_xor_12: {},", log_of("bitwise_xor_10_0")).unwrap();
    writeln!(w, "    verify_bitwise_xor_4: {},", log_of("bitwise_xor_4_0")).unwrap();
    writeln!(w, "    verify_bitwise_xor_7: {},", log_of("bitwise_xor_7_0")).unwrap();
    writeln!(w, "    verify_bitwise_xor_9: {},", log_of("bitwise_xor_9_0")).unwrap();
    writeln!(w, "    range_check_16: {},", log_of("seq_16")).unwrap();
    writeln!(w, "}};").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// Per-column log sizes of the multiverifier circuit's preprocessed trace,")
        .unwrap();
    writeln!(w, "/// in size-sorted column order — the same order as the index constants in")
        .unwrap();
    writeln!(w, "/// `crate::preprocessed_columns`. Every column of a component shares that")
        .unwrap();
    writeln!(w, "/// component's log size, so each entry references the owning component's")
        .unwrap();
    writeln!(w, "/// `COMPONENT_LOG_SIZES` field.").unwrap();
    writeln!(w, "pub const PREPROCESSED_COLUMN_LOG_SIZES: [u32; {}] = [", layout.len()).unwrap();
    for (i, (id, _)) in layout.iter().enumerate() {
        let sep = if i + 1 == layout.len() { "" } else { "," };
        writeln!(w, "    COMPONENT_LOG_SIZES.{}{sep} // {id}", cairo_component(id)).unwrap();
    }
    writeln!(w, "];").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "{END_MARKER}").unwrap();
    out
}

/// Renders the generated section of `preprocessed_columns.cairo`: the column count and one `*_IDX`
/// constant per column, in commitment order.
fn render_preprocessed_columns(target_sizes: &ComponentSizes) -> String {
    let layout = layout(target_sizes);
    let mut out = String::new();
    let w = &mut out;
    writeln!(w, "{BEGIN_MARKER}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "pub const NUM_PREPROCESSED_COLUMNS: u32 = {};", layout.len()).unwrap();

    let mut idx = 0;
    while idx < layout.len() {
        let (id, log_size) = &layout[idx];
        let component = cairo_component(id);
        let group_end = layout[idx..]
            .iter()
            .position(|(other, _)| cairo_component(other) != component)
            .map(|len| idx + len)
            .unwrap_or(layout.len());
        writeln!(w).unwrap();
        if component == "qm31_ops" {
            writeln!(
                w,
                "// qm31_ops_* (log_size={log_size}). Hand-ported components reference these \
                 without the"
            )
            .unwrap();
            writeln!(
                w,
                "// `qm31_ops_` prefix and with legacy names (`OP_0/OP_1/DST` for `in0/in1/out`)."
            )
            .unwrap();
        }
        for (id, _) in &layout[idx..group_end] {
            writeln!(w, "pub const {}: PreprocessedColumnIdx = {idx};", idx_const_name(id))
                .unwrap();
            idx += 1;
        }
    }
    writeln!(w).unwrap();
    writeln!(w, "{END_MARKER}").unwrap();
    out
}

/// Replaces (or checks) the generated section between the markers in `file_name`.
fn assert_generated_section(file_name: &str, rendered: &str) {
    let path = circuit_air_src().join(file_name);
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    let begin = committed
        .find(BEGIN_MARKER)
        .unwrap_or_else(|| panic!("{file_name} is missing the BEGIN GENERATED marker"));
    let end = committed
        .find(END_MARKER)
        .unwrap_or_else(|| panic!("{file_name} is missing the END GENERATED marker"))
        + END_MARKER.len();
    assert!(begin < end, "{file_name}: markers out of order");

    if std::env::var("FIX").is_ok() {
        let fixed = format!("{}{}{}", &committed[..begin], rendered.trim_end(), &committed[end..]);
        std::fs::write(&path, fixed)
            .unwrap_or_else(|err| panic!("cannot write {}: {err}", path.display()));
        return;
    }
    assert_eq!(
        &committed[begin..end],
        rendered.trim_end(),
        "{file_name}'s generated section does not match the production registry; run this test \
         with FIX=1 to regenerate",
    );
}

/// The Cairo verifier's pinned PCS config, component sizes and column layout must match the
/// production registry — the root proof it verifies is produced under its config. Derives the
/// shared target by building the circuit topologies (traces 25-29; no commitments and no
/// preprocessed-trace values, so this is fast).
#[test]
fn test_cairo_verifier_consts_match_production_registry() {
    let production = RegistryDefinition::load(&repo_root(), "production");
    let target_sizes = production.shared_target_sizes();
    let fri_config = production.circuit_fri_config();
    assert_generated_section(
        "multiverifier_consts.cairo",
        &render_multiverifier_consts(fri_config, &target_sizes),
    );
    assert_generated_section(
        "preprocessed_columns.cairo",
        &render_preprocessed_columns(&target_sizes),
    );

    // The canonical_small registry pads to the production shape, so that its goldens' root proof —
    // the Cairo verifier's execution fixture — is verified by exactly these consts.
    let small = RegistryDefinition::load(&repo_root(), "canonical_small");
    assert_eq!(
        small.pad_to_component_log_sizes.as_ref(),
        Some(&circuit_registry::LogSizes::from(&target_sizes)),
        "canonical_small's pad_to_component_log_sizes must equal the production target"
    );
    assert_eq!(
        small.circuit_fri_config(),
        fri_config,
        "canonical_small's circuit FRI config must equal production's"
    );
}
