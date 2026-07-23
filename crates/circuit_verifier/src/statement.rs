use circuits::blake::{HashValue, unpack_qm31s_to_u32_words};
use circuits::context::{Context, U_VAR_IDX, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits::simd::Simd;
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::logup::logup_use_term;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::statement::Statement;
use indexmap::IndexMap;
use itertools::{Itertools, chain, zip_eq};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::circuit_components::PerComponent;
use crate::circuit_hash::compute_circuit_hash;
use crate::components::eq::CircuitEqComponent;
use crate::components::qm31_ops::CircuitQm31OpsComponent;
use crate::components::{
    blake_g_gate, m_31_to_u_32, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use crate::relations::GATE_RELATION_ID;
use crate::verify::CircuitConfig;

// TODO(ilya): Update this to the correct values.
pub const INTERACTION_POW_BITS: u32 = 20;

pub struct CircuitStatement<Value: IValue> {
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    /// The number of output gates, excluding the wire of `u` (index 2).
    n_outputs: usize,
    /// The values of the output gates.
    pub output_values: Vec<Var>,
    /// Per-component trace log sizes packed as a [`Simd`].
    pub component_log_sizes: Simd,
    /// Maps preprocessed column ids to their log sizes.
    /// The order of the keys is the same as the order of the columns in the prover's preprocessed
    /// trace.
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    /// The preprocessed trace root.
    pub preprocessed_root: HashValue<Var>,
    /// The circuit hash.
    pub circuit_hash: HashValue<Var>,
}
impl<Value: IValue> CircuitStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        circuit_config: &CircuitConfig,
        output_values: &[Var],
    ) -> Self {
        let CircuitConfig { config, n_outputs, preprocessed_column_log_sizes, preprocessed_root } =
            circuit_config;
        assert_eq!(output_values.len(), *n_outputs);
        let output_values = output_values.to_vec();
        // Guess the preprocessed root. The guessed wires enter the hash that will be output by
        // this verifier. To ensure soundness in a recursive setup, it is *critical* that this hash
        // is reconstructed by the last verifier, which we can assume honest.
        let preprocessed_root = HashValue(std::array::from_fn(|i| {
            U32Wrapper::new_unsafe(Value::from_qm31(*preprocessed_root[i].get()))
        }))
        .guess(context);

        // Order the components by ascending trace log size — the committed-column order. stwo
        // commits the trace/interaction columns sorted by size (see
        // `stwo::prover::vcs_lifted::prover::MerkleProverLifted::commit`), so iterating the
        // components in this order makes the natural component order coincide with the
        // committed column order. That lets `sorting_required` return `false`, skipping the
        // in-circuit query-column sort during decommitment.
        let components = all_circuit_components::<Value>();
        let log_sizes = circuit_component_log_sizes(&components, preprocessed_column_log_sizes);

        // The circuit hash mixes component log sizes by `COMPONENT_NAMES` order, so it is
        // independent of the sort below; compute it from the unsorted map.
        let circuit_hash = compute_circuit_hash(
            context,
            &log_sizes,
            config.fri_config.log_blowup_factor,
            &preprocessed_root,
        );

        let (sorted_components, sorted_log_sizes): (Vec<_>, Vec<_>) =
            zip_eq(components, log_sizes.into_iter().map(|(_, log_size)| log_size))
                .sorted_by_key(|(_, log_size)| *log_size)
                .unzip();
        let components = IndexMap::from_iter(sorted_components);

        let n_components = sorted_log_sizes.len();
        let packed_log_sizes = pack_into_qm31s(sorted_log_sizes.iter().cloned())
            .into_iter()
            .map(|qm31| context.constant(qm31))
            .collect_vec();
        let component_log_sizes = Simd::from_packed(packed_log_sizes, n_components);

        Self {
            components,
            n_outputs: *n_outputs,
            output_values,
            component_log_sizes,
            preprocessed_column_log_sizes: preprocessed_column_log_sizes.clone(),
            preprocessed_root,
            circuit_hash,
        }
    }
}

impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<U32Wrapper<Var>>> {
        let circuit_hash_words = self.circuit_hash.iter().copied().collect_vec();
        let output_words = unpack_qm31s_to_u32_words(context, self.output_values.iter().copied());
        vec![circuit_hash_words, output_words]
    }

    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
        &self.components
    }

    fn get_component_log_sizes(&self) -> &Simd {
        &self.component_log_sizes
    }

    /// The circuit components are committed and iterated in size-sorted order (see the sort in
    /// `CircuitStatement::new`), so the trace and interaction query columns are already in
    /// committed order and need no sorting during decommitment.
    fn sorting_required(&self) -> bool {
        false
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();

        // Output gates public logup sum contribution.
        let gate_relation_id = context.constant(GATE_RELATION_ID.into());
        // Construct the output addresses. They are located at addresses `[3, 3 + n_outputs)`.
        let output_addresses = ((U_VAR_IDX + 1)..(U_VAR_IDX + 1 + self.n_outputs))
            .map(|addr| M31Wrapper::new_unsafe(context.constant(addr.into())))
            .collect_vec();
        // Add the pair `(U_VAR_IDX, U_VALUE)` to the addresses and values, respectively.
        let u_addr = M31Wrapper::new_unsafe(context.constant(U_VAR_IDX.into()));
        let u_val = context.u();
        let output_addresses = chain!(&output_addresses, [&u_addr]);
        let output_values = chain!(&self.output_values, [&u_val]);

        for (output_address, output_value) in zip_eq(output_addresses, output_values) {
            let [output_value_0, output_value_1, output_value_2, output_value_3] =
                Simd::unpack(context, &Simd::from_packed(vec![*output_value], 4))
                    .try_into()
                    .unwrap();
            let term = logup_use_term(
                context,
                &[
                    gate_relation_id,
                    *output_address.get(),
                    output_value_0,
                    output_value_1,
                    output_value_2,
                    output_value_3,
                ],
                interaction_elements,
            );
            sum = eval!(context, (sum) + (term));
        }

        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_column_log_sizes.keys().cloned().collect()
    }

    fn get_preprocessed_root(&self, _context: &mut Context<Value>) -> HashValue<Var> {
        self.preprocessed_root.clone()
    }
}

pub fn all_circuit_components<Value: IValue>() -> IndexMap<&'static str, Box<dyn CircuitEval<Value>>>
{
    let components = PerComponent::<Box<dyn CircuitEval<Value>>> {
        eq: Box::new(CircuitEqComponent {}),
        qm31_ops: Box::new(CircuitQm31OpsComponent {}),
        triple_xor: Box::new(triple_xor::Component {}),
        m_31_to_u_32: Box::new(m_31_to_u_32::Component {}),
        blake_g_gate: Box::new(blake_g_gate::Component {}),
        verify_bitwise_xor_8: Box::new(verify_bitwise_xor_8::Component {}),
        verify_bitwise_xor_12: Box::new(verify_bitwise_xor_12::Component {}),
        verify_bitwise_xor_4: Box::new(verify_bitwise_xor_4::Component {}),
        verify_bitwise_xor_7: Box::new(verify_bitwise_xor_7::Component {}),
        verify_bitwise_xor_9: Box::new(verify_bitwise_xor_9::Component {}),
        range_check_16: Box::new(range_check_16::Component {}),
    };
    IndexMap::from_iter(components.into_named_iter())
}

/// Resolves the (static) log size of every circuit component, keyed by component name.
pub fn circuit_component_log_sizes<Value: IValue>(
    components: &IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
) -> OrderedHashMap<&'static str, u32> {
    components
        .iter()
        .map(|(name, c)| {
            let log_size = c
                .log_size(preprocessed_column_log_sizes)
                .expect("The circuit components can't have a dynamic log_size.");
            (*name, log_size)
        })
        .collect()
}

/// Builds the [`ProofConfig`] for the circuit verifier circuit, ordering the components by
/// ascending trace log size (the column layout the circuit verifier expects).
pub fn circuit_verifier_proof_config(
    preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    pcs_config: &PcsConfig,
) -> ProofConfig {
    let mut components = all_circuit_components::<QM31>();
    let log_sizes = circuit_component_log_sizes(&components, preprocessed_column_log_sizes);
    components.sort_by_key(|a, _| log_sizes[*a]);
    ProofConfig::new(
        &components,
        preprocessed_column_log_sizes.len(),
        pcs_config,
        INTERACTION_POW_BITS,
    )
}
