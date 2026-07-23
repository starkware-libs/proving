use std::collections::HashMap;

use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::simd::Simd;
use circuits::wrappers::U32Wrapper;
use indexmap::IndexMap;
use stwo::core::circle::CirclePoint;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::constraint_eval::CircuitEval;
use crate::proof::InteractionAtOods;

/// Values at the OODS point (and its previous point where applicable).
pub struct OodsSamples<'a> {
    pub preprocessed_columns: &'a [Var],
    pub trace: &'a [Var],
    pub interaction: &'a [InteractionAtOods<Var>],
}

pub struct EvaluateArgs<'a> {
    pub oods_samples: OodsSamples<'a>,
    pub pt: CirclePoint<Var>,
    pub log_domain_size: usize,
    pub composition_polynomial_coeff: Var,
    pub interaction_elements: [Var; 2],
    pub claimed_sums: &'a [Var],
    pub component_sizes: &'a [Var],
    pub n_instances_bits: &'a [Simd],
}

/// Represents an AIR and its public inputs.
pub trait Statement<Value: IValue> {
    /// Returns the statement's public inputs to mix into the channel, one `mix_u32s` call per
    /// inner `Vec`, in the exact order (and with the exact grouping) the prover mixed them.
    /// The concrete contents are statement-specific.
    ///
    /// Each value is a `u32` word: a QM31 felt is encoded as its four coordinate words (matching
    /// the little-endian byte stream `mix_u32s` feeds to Blake2s), so this produces the same
    /// transcript as mixing the felts directly.
    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<U32Wrapper<Var>>>;

    /// Returns the AIR components that define the constraint system.
    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>>;

    fn get_component_log_sizes(&self) -> &Simd;

    /// Returns the IDs of the preprocessed columns used by this statement's components.
    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId>;

    /// Returns the expected preprocessed trace root as circuit variables.
    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var>;

    /// Returns the part of the logup sum determined by the public statement.
    fn public_logup_sum(&self, context: &mut Context<Value>, interaction_elements: [Var; 2])
    -> Var;

    /// Returns statement-specific named parameters passed to component constraint evaluators.
    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
        HashMap::new()
    }

    /// Whether the trace and interaction query columns must be sorted by (log size,
    /// column index) before Merkle leaf hashing during decommitment, to match the order
    /// in which those columns were committed to the Merkle tree. Returns `true` by
    /// default, matching the standard verifier behavior. Statements whose column order
    /// already matches the committed order can override this to `false` to skip sorting.
    fn sorting_required(&self) -> bool {
        true
    }

    /// Performs statement-level consistency checks on the claim.
    fn verify_claim(
        &self,
        _context: &mut Context<Value>,
        _component_sizes: &[Var],
        _shifted_relation_uses: &HashMap<String, Var>,
    ) {
    }
}
