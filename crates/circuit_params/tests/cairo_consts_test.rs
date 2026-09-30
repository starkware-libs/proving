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

use circuit_cairo_verifier::utils::load_program;
use circuit_common::finalize::{ComponentSizes, compute_padded_sizes};
use circuit_common::preprocessed::{PreprocessedCircuit, layout_from_component_sizes};
use circuit_params::{CircuitsBuilder, RegistryDefinition, padded_shared_target};
use circuits::blake::HashValue;
use stwo::core::fri::FriParams;

const BEGIN_MARKER: &str =
    "// === BEGIN GENERATED (see cairo_consts_test.rs; running it with FIX=1 regenerates) ===";
const END_MARKER: &str = "// === END GENERATED ===";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `definition`'s shared padding target and the multiverifier padded to it: the elementwise max
/// over the leaf circuits of its trace range, closed under the multiverifier fixpoint.
///
/// Builds each leaf's topology with a dummy Cairo root, which the padded sizes do not depend on.
/// That keeps this test cheap — the binary commits the real roots instead, at a cost this test
/// does not want to pay per run.
// TODO(yair): Consider using the real roots, as the binary does, and moving this test to
// `slow-tests`: a dummy cannot guarantee the same target sizes, however unlikely a difference.
fn shared_target(definition: &RegistryDefinition) -> (ComponentSizes, PreprocessedCircuit) {
    let cairo_params = definition.cairo_params();
    let circuits_builder = CircuitsBuilder {
        cairo_preprocessed_trace_variant: cairo_params.preprocessed_trace,
        leaf_program: load_program(&definition.program),
        cairo_fri_params: cairo_params.fri_params,
        circuit_fri_params: definition.circuit_fri_params(),
        add_zk_blinding: definition.add_zk_blinding,
    };
    let leaves_max_sizes = (definition.min_trace_log_size..=definition.max_trace_log_size)
        .map(|trace_log_size| {
            let dummy_root = HashValue::from([0; 8]);
            let context = circuits_builder.build_leaf_context(trace_log_size, dummy_root);
            compute_padded_sizes(&context)
        })
        .reduce(|a, b| a.elementwise_max(&b))
        .expect("the trace range is non-empty");
    padded_shared_target(
        leaves_max_sizes,
        definition.circuit_fri_params(),
        definition.pad_to_component_log_sizes.as_ref(),
    )
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
        _ if id.starts_with("qm_31_ops_") => "qm_31_ops",
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

/// The Cairo `*_IDX` constant name of a preprocessed column id. The AIR-infra naming differs
/// from the registry ids for some columns, aliased here (`m31_to_u32/m_31_to_u_32`, `f0/f_0`).
fn idx_const_name(id: &str) -> String {
    let renamed = match id {
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
fn render_multiverifier_consts(fri_params: FriParams, target_sizes: &ComponentSizes) -> String {
    let layout = layout(target_sizes);
    let log_of = |id: &str| {
        layout.iter().find(|(col, _)| col == id).unwrap_or_else(|| panic!("missing {id}")).1
    };
    let FriParams {
        pow_bits,
        log_blowup_factor,
        log_last_layer_degree_bound,
        n_queries,
        fold_step,
    } = fri_params;

    let mut out = String::new();
    let w = &mut out;
    writeln!(w, "{BEGIN_MARKER}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// Expected FRI params of the multiverifier circuit's proof.").unwrap();
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
    writeln!(w, "pub const CIRCUIT_FRI_PARAMS: FriParams = FriParams {{").unwrap();
    writeln!(
        w,
        "    pow_bits: {pow_bits}, log_blowup_factor: {log_blowup_factor}, \
         log_last_layer_degree_bound: {log_last_layer_degree_bound}, n_queries: {n_queries}, \
         fold_step: {fold_step},"
    )
    .unwrap();
    writeln!(w, "}};").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// Each component's log size.").unwrap();
    writeln!(w, "pub const COMPONENT_LOG_SIZES: PerComponent<u32> = PerComponent {{").unwrap();
    writeln!(w, "    eq: {},", log_of("eq_in0_address")).unwrap();
    writeln!(w, "    qm_31_ops: {},", log_of("qm_31_ops_add_flag")).unwrap();
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
    writeln!(
        w,
        "/// Log degree bound of the circuit's trace, equal to the largest preprocessed column \
         log size."
    )
    .unwrap();
    writeln!(
        w,
        "pub const TRACE_LOG_DEGREE_BOUND: u32 = {};",
        layout.iter().map(|(_, log_size)| *log_size).max().expect("empty preprocessed layout")
    )
    .unwrap();
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
        if component == "qm_31_ops" {
            writeln!(w, "// qm_31_ops_* (log_size={log_size}).").unwrap();
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
/// shared target by building the circuit topologies (traces 25-29; no commitments, so this is
/// fast).
#[test]
fn test_cairo_verifier_consts_match_production_registry() {
    let production = RegistryDefinition::load(&repo_root(), "production");
    // TODO(yair): Add a sizes-only shared-target variant; the discarded multiverifier's
    // preprocessed trace is the bulk of this test's memory.
    let (target_sizes, _preprocessed_multiverifier) = shared_target(&production);
    let fri_params = production.circuit_fri_params();
    assert_generated_section(
        "multiverifier_consts.cairo",
        &render_multiverifier_consts(fri_params, &target_sizes),
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
        small.circuit_fri_params(),
        fri_params,
        "canonical_small's circuit FRI params must equal production's"
    );
}
