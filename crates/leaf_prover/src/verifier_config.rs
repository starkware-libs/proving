//! The configuration of the leaf verifier circuit: the circuit that verifies a leaf's Cairo proof.

use std::sync::Arc;

use cairo_air::verifier::INTERACTION_POW_BITS;
use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::verify::CairoVerifierConfig;
use circuits::blake::HashValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::ProofConfig;
use indexmap::IndexMap;
use itertools::Itertools;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;
use stwo_cairo_common::prover_types::cpu::M31;
use stwo_cairo_prover::witness::prelude::QM31;

/// The components of the circuit that verifies the Cairo proof.
pub struct LeafVerifierComponents {
    /// Map from component name to the circuit evaluator that verifies it.
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<QM31>>>,
    /// One bit per possible component: `true` if the component is enabled (present).
    pub enabled_bits: Vec<bool>,
}

/// Create a configuration used to build the leaf verifier circuit
///
/// `preprocessed_trace_variant` - The preprocessed trace of the Cairo programs the circuit will
///     verify.
/// `pcs_config` - The PCS config used to produce the proofs the circuit will verify.
/// `cairo_component_names` - The components that should be in the Cairo AIR. The circuit
///     will only be able to verify proofs using all of these components and no extra components.
/// `program` - The circuit will verify executions of this program.
/// `preprocessed_root` - The root of the preprocessed trace of the Cairo proofs to be verified.
/// `zk_blinding_amount` - How many gates to add as padding to the circuit. None turns
///     off ZK blinding.
pub fn leaf_verifier_config(
    preprocessed_trace_variant: PreProcessedTraceVariant,
    pcs_config: &PcsConfig,
    cairo_component_names: &[String],
    program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    preprocessed_root: HashValue<QM31>,
    zk_blinding_amount: Option<usize>,
) -> CairoVerifierConfig {
    let LeafVerifierComponents { components, enabled_bits } =
        leaf_verifier_components(cairo_component_names);
    let proof_config = ProofConfig::new(
        &components,
        preprocessed_trace_variant.n_columns(),
        pcs_config,
        INTERACTION_POW_BITS,
    );
    CairoVerifierConfig {
        proof_config,
        enabled_bits,
        program,
        preprocessed_root,
        preprocessed_trace_variant,
        zk_blinding_amount,
    }
}

/// Creates the component list and enabled bits for the circuit that verifies the Cairo proof.
/// The set of components is constant to keep the verifier circuit stable. The trace is expected to
/// contain all the components in this set.
fn leaf_verifier_components(component_names: &[String]) -> LeafVerifierComponents {
    let mut components: IndexMap<&'static str, Box<dyn CircuitEval<QM31>>> = IndexMap::default();
    let mut enabled_bits = vec![];
    let all_components = all_components::<QM31>();
    for name in component_names {
        assert!(all_components.contains_key(name.as_str()), "Unknown component {name}");
    }
    assert!(
        component_names.iter().all_unique(),
        "Duplicate component names in {:?}",
        component_names
    );
    for (name, component) in all_components {
        if component_names.contains(&name.into()) {
            components.insert(name, component);
            enabled_bits.push(true);
        } else {
            enabled_bits.push(false);
        }
    }
    LeafVerifierComponents { components, enabled_bits }
}
