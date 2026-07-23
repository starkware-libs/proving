use std::array;
use std::collections::HashMap;
use std::sync::Arc;

use cairo_air::components::memory_address_to_id::MEMORY_ADDRESS_TO_ID_SPLIT;
use cairo_air::flat_claims::FlatClaim;
use cairo_air::relations::{
    MEMORY_ADDRESS_TO_ID_RELATION_ID, MEMORY_ID_TO_BIG_RELATION_ID, OPCODES_RELATION_ID,
};
use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s, m31_to_u32};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::IValue;
use circuits::ops::{Guess, eq};
use circuits::simd::Simd;
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::logup::logup_use_term;
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::statement::Statement;
use circuits_stark_verifier::verify::RELATION_USES_NUM_ROWS_SHIFT;
use indexmap::IndexMap;
use itertools::{Itertools, chain, zip_eq};
use stwo::core::fields::m31::{M31, P as M31_P};
use stwo::core::fields::qm31::QM31;
use stwo_cairo_common::builtins::{
    ADD_MOD_BUILTIN_MEMORY_CELLS, BITWISE_BUILTIN_MEMORY_CELLS, EC_OP_BUILTIN_MEMORY_CELLS,
    ECDSA_MEMORY_CELLS, KECCAK_MEMORY_CELLS, MUL_MOD_BUILTIN_MEMORY_CELLS,
    PEDERSEN_BUILTIN_MEMORY_CELLS, POSEIDON_BUILTIN_MEMORY_CELLS,
    RANGE_CHECK_96_BUILTIN_MEMORY_CELLS, RANGE_CHECK_BUILTIN_MEMORY_CELLS,
};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    MAX_SEQUENCE_LOG_SIZE, PreProcessedTraceVariant,
};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::verify::enabled_components;

#[cfg(test)]
#[path = "statement_test.rs"]
pub mod test;

const N_SEGMENTS: usize = 11;
const N_SAFE_CALL_IDS: usize = 2;

// A memory value is stored as 28 9bit limbs.
pub const MEMORY_VALUES_LIMBS: usize = 28;

/// Number of public output cells. The program emits its output as a single Blake2s digest split
/// into two memory cells, each holding one 128-bit half of the digest.
pub const N_OUTPUTS: usize = 2;
const PUB_MEMORY_VALUE_M31_LEN: usize = 2;
const STATE_LEN: usize = 3;
/// Length of the fixed-size portion of the serialized auxiliary data: the initial/final states,
/// segment ranges, safe-call ids and the [`N_OUTPUTS`] output cells. The variable-size portion
/// (program and component log sizes) is added on top by the caller.
pub const AUX_DATA_FIXED_LEN: usize =
    2 * STATE_LEN + 2 * PUB_MEMORY_VALUE_M31_LEN * N_SEGMENTS + N_SAFE_CALL_IDS + N_OUTPUTS;

const LIMB_BITS: usize = 9;

/// Number of 32-bit words in a 128-bit output cell value, i.e. how many `u32`s fit in a `u128`.
/// The program's output is a Blake2s digest ([`BLAKE2S_DIGEST_N_WORDS`] words) split across
/// [`N_OUTPUTS`] cells, one 128-bit half each.
pub const N_WORDS_PER_OUTPUT_CELL: usize = u128::BITS as usize / u32::BITS as usize;
// The whole digest must map exactly onto the output cells.
const _: () = assert!(N_OUTPUTS * N_WORDS_PER_OUTPUT_CELL == BLAKE2S_DIGEST_N_WORDS);
const MEMORY_ADDRESS_BITS: u32 = 29;
const BUILTIN_USAGE_BITS: u32 = 27;

pub struct CasmState<T> {
    pub pc: T,
    pub ap: T,
    pub fp: T,
}
impl CasmState<Var> {
    pub fn logup_term(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let Self { pc, ap, fp } = self;
        let elements = [context.constant(OPCODES_RELATION_ID.into()), *pc, *ap, *fp];
        logup_use_term(context, &elements, interaction_elements)
    }
}

// A public memory value that fits in 29 bits.
pub struct PubMemoryAddress<T> {
    pub id: T,
    pub value: T,
}

pub fn split_address_to_9bit_limbs(context: &mut Context<impl IValue>, value: Var) -> [Var; 4] {
    let simd = Simd::from_packed(vec![value], 1);
    let extracted_bits = extract_bits(context, &simd, MEMORY_ADDRESS_BITS);

    let mut limbs_iter = extracted_bits.chunks(LIMB_BITS).map(|limb_bits| {
        let limb = Simd::combine_bits(context, limb_bits);
        Simd::unpack(context, &limb)[0]
    });
    array::from_fn(|_| limbs_iter.next().unwrap())
}

pub struct SegmentRange<T> {
    pub start: PubMemoryAddress<T>,
    pub end: PubMemoryAddress<T>,
}

// Auxiliary data that is used for verifying an execution of a Cairo program.
pub struct AuxData {
    // TODO(ilya): Use `M31Wrapper<Var>` for all fields.
    pub initial_state: CasmState<Var>,
    pub final_state: CasmState<Var>,
    pub segment_ranges: [SegmentRange<Var>; N_SEGMENTS],
    pub safe_call_ids: [Var; 2],
    pub output_ids: Vec<Var>,
    pub program_ids: Vec<Var>,
    pub component_log_sizes: Vec<M31Wrapper<Var>>,
}

impl AuxData {
    /// Parses the auxiliary data from a slice of variables.
    ///
    /// `data` is laid out as the concatenation of, in order: the fixed-size fields
    /// (initial state, final state, segment ranges and safe-call ids), then
    /// `output_ids`, `program_ids` and `component_log_sizes`.
    pub fn parse_from_vars(data: &[Var], program_len: usize, n_components: usize) -> Self {
        let mut iter = data.iter();

        let initial_state = CasmState {
            pc: *iter.next().unwrap(),
            ap: *iter.next().unwrap(),
            fp: *iter.next().unwrap(),
        };
        let final_state = CasmState {
            pc: *iter.next().unwrap(),
            ap: *iter.next().unwrap(),
            fp: *iter.next().unwrap(),
        };

        let segment_ranges = array::from_fn(|_| SegmentRange {
            start: PubMemoryAddress { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
            end: PubMemoryAddress { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
        });

        let safe_call_ids = [*iter.next().unwrap(), *iter.next().unwrap()];
        let output_ids = iter.by_ref().take(N_OUTPUTS).cloned().collect_vec();
        let program_ids = iter.by_ref().take(program_len).cloned().collect_vec();
        let component_log_sizes = iter.map(|v| M31Wrapper::new_unsafe(*v)).collect_vec();
        assert_eq!(component_log_sizes.len(), n_components);

        Self {
            initial_state,
            final_state,
            segment_ranges,
            safe_call_ids,
            output_ids,
            program_ids,
            component_log_sizes,
        }
    }
}

// Serialize a claim into the format expected by CairoStatement::new
pub fn serialize_aux_data(claim: &FlatClaim) -> Vec<M31> {
    let mut result = vec![];

    for state in [&claim.public_data.initial_state, &claim.public_data.final_state] {
        result.push(state.pc);
        result.push(state.ap);
        result.push(state.fp);
    }

    let public_memory = &claim.public_data.public_memory;

    for segment in public_memory.public_segments.present_segments().iter() {
        result.push(segment.start_ptr.id.into());
        result.push(segment.start_ptr.value.into());
        result.push(segment.stop_ptr.id.into());
        result.push(segment.stop_ptr.value.into());
    }

    result.extend(public_memory.safe_call_ids.iter().map(|id| M31::from(*id)));
    result.extend(public_memory.output.iter().map(|(id, _value)| M31::from(*id)));
    result.extend(public_memory.program.iter().map(|(id, _value)| M31::from(*id)));
    result.extend(claim.component_log_sizes.iter().map(|size| M31::from(*size)));

    result
}

pub struct CairoStatement<Value: IValue> {
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    /// One flag per component in the full list of components (in the order returned by
    /// `all_components()`), indicating whether it is enabled. Mixed into the channel by
    /// `claims_to_mix` for compatibility with the Cairo1 verifier, where the set of components in
    /// the AIR can be set dynamically.
    pub enabled_bits: Vec<bool>,
    pub aux_data: AuxData,
    pub packed_component_log_sizes: Simd,
    pub program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    pub outputs: Vec<[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]>,
    pub preprocessed_root: HashValue<QM31>,
    pub preprocessed_trace_variant: PreProcessedTraceVariant,
}

impl<Value: IValue> CairoStatement<Value> {
    /// Verifies the builtins.
    ///
    /// Assumes that the start and end addresses of the segment ranges are less than 2^29 (this is
    /// guaranteed by `segment_ranges_logup_sum`).
    pub fn verify_builtins(&self, context: &mut Context<Value>, component_sizes: &[Var]) {
        let [
            output_segment_range,
            pedersen_segment_range,
            range_check_128_segment_range,
            ecdsa_segment_range,
            bitwise_segment_range,
            ec_op_segment_range,
            keccak_segment_range,
            poseidon_segment_range,
            range_check96_segment_range,
            add_mod_segment_range,
            mul_mod_segment_range,
        ] = &self.aux_data.segment_ranges;

        // Validate the output segment range.
        let diff =
            eval!(context, (output_segment_range.end.value) - (output_segment_range.start.value));
        let n_outputs = context.constant(self.outputs.len().into());
        eq(context, diff, n_outputs);

        let builtin_segment_ranges = [
            pedersen_segment_range,
            range_check_128_segment_range,
            bitwise_segment_range,
            poseidon_segment_range,
            ec_op_segment_range,
            ecdsa_segment_range,
            keccak_segment_range,
            range_check96_segment_range,
            add_mod_segment_range,
            mul_mod_segment_range,
        ];
        let start_addresses = Simd::pack(
            context,
            &builtin_segment_ranges
                .iter()
                .map(|segment_range| M31Wrapper::new_unsafe(segment_range.start.value))
                .collect_vec(),
        );
        let end_addresses = Simd::pack(
            context,
            &builtin_segment_ranges
                .iter()
                .map(|segment_range| M31Wrapper::new_unsafe(segment_range.end.value))
                .collect_vec(),
        );
        let diff = Simd::sub(context, &end_addresses, &start_addresses);

        // Select the pedersen builtin based on the trace variant, for `CanonicalWithoutPedersen`
        // the builtin is can't be used so we can use either of the two.
        let pedersen_builtin = match self.preprocessed_trace_variant {
            PreProcessedTraceVariant::CanonicalSmall => "pedersen_builtin_narrow_windows",
            PreProcessedTraceVariant::Canonical
            | PreProcessedTraceVariant::CanonicalWithoutPedersen => "pedersen_builtin",
        };

        let builtin_instance_sizes = [
            (pedersen_builtin, PEDERSEN_BUILTIN_MEMORY_CELLS),
            ("range_check_builtin", RANGE_CHECK_BUILTIN_MEMORY_CELLS),
            ("bitwise_builtin", BITWISE_BUILTIN_MEMORY_CELLS),
            ("poseidon_builtin", POSEIDON_BUILTIN_MEMORY_CELLS),
            ("ec_op_builtin", EC_OP_BUILTIN_MEMORY_CELLS),
            ("ecdsa_builtin", ECDSA_MEMORY_CELLS),
            ("keccak_builtin", KECCAK_MEMORY_CELLS),
            ("range_check96_builtin", RANGE_CHECK_96_BUILTIN_MEMORY_CELLS),
            ("add_mod_builtin", ADD_MOD_BUILTIN_MEMORY_CELLS),
            ("mul_mod_builtin", MUL_MOD_BUILTIN_MEMORY_CELLS),
        ];
        assert_eq!(builtin_instance_sizes.len(), builtin_segment_ranges.len());
        let builtin_instance_size_inverses = pack_into_qm31s(
            builtin_instance_sizes.iter().map(|(_name, size)| M31::from(*size).inverse()),
        )
        .into_iter()
        .map(|qm31| context.constant(qm31))
        .collect();
        let packed_instance_size_inverses =
            Simd::from_packed(builtin_instance_size_inverses, builtin_segment_ranges.len());

        let n_uses_simd = Simd::mul(context, &diff, &packed_instance_size_inverses);

        // Range-check the number of times each builtin is used.
        // n_uses = (end - start) / instance_size, which implies end = start + n_uses *
        // instance_size (mod M31_P). Since end and start are 29 bits and n_uses is 27 bits,
        // start + n_uses * instance_size < 2^29 + 2^27 * max_supported_builtin_instance_size <
        // M31_P, so this equality also holds over the integers.
        // Note that there is an assertion involving max_supported_builtin_instance_size that
        // guarantees this doesn't overflow.
        extract_bits(context, &n_uses_simd, BUILTIN_USAGE_BITS);

        let actual_uses_iter = Simd::unpack(context, &n_uses_simd).into_iter();
        let mut range_checks = vec![];
        let mut max_supported_builtin_instance_size = 0_usize;
        for ((name, size), actual_uses) in zip_eq(builtin_instance_sizes, actual_uses_iter) {
            let Some(index) = self.components.get_index_of(name) else {
                // The component is not supported by the circuit - actual_uses must be 0.
                eq(context, actual_uses, context.zero());
                continue;
            };

            if size > max_supported_builtin_instance_size {
                max_supported_builtin_instance_size = size;
            }

            let component_size = component_sizes[index];
            // Check that 0 <= component_size - actual_uses < 2^27 => actual_uses <= component_size.
            let diff = eval!(context, (component_size) - (actual_uses));
            range_checks.push(M31Wrapper::new_unsafe(diff));
        }
        assert!(
            (1 << MEMORY_ADDRESS_BITS)
                + (1 << BUILTIN_USAGE_BITS) * max_supported_builtin_instance_size
                < usize::try_from(M31_P).unwrap(),
        );

        let rc_simd = Simd::pack(context, &range_checks);
        extract_bits(context, &rc_simd, BUILTIN_USAGE_BITS);
    }
}

/// Returns the little-endian bit decomposition (LSB first) of a 32-bit `word`, as 32 single-bit
/// [`Simd`]s. The word is stored as two 16-bit limbs `(low, high, 0, 0)`; each is decomposed and
/// range-checked to 16 bits by [`extract_bits`].
fn word_to_le_bits<Value: IValue>(
    context: &mut Context<Value>,
    word: &U32Wrapper<Var>,
) -> Vec<Simd> {
    let packed = Simd::from_packed(vec![*word.get()], 2);
    let low = Simd::from_packed(vec![Simd::unpack_idx(context, &packed, 0)], 1);
    let high = Simd::from_packed(vec![Simd::unpack_idx(context, &packed, 1)], 1);
    chain!(extract_bits(context, &low, 16), extract_bits(context, &high, 16)).collect()
}

/// Splits the digest `hash` into its 2 u128s, each decomposed into nine-bit limbs (little-endian).
fn output_limbs_from_hash<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<Var>,
) -> Vec<[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]> {
    let zero = M31Wrapper::new_unsafe(context.zero());
    let mut outputs = Vec::with_capacity(N_OUTPUTS);
    for half in hash.chunks(N_WORDS_PER_OUTPUT_CELL) {
        // Little-endian bit stream of the cell's 128-bit value (LSB first).
        let mut bits = Vec::with_capacity(N_WORDS_PER_OUTPUT_CELL * 32);
        for word in half {
            bits.extend(word_to_le_bits(context, word));
        }
        // Regroup the bits into nine-bit limbs: the 128 bits fill the low div_ceil(128, 9) = 15
        // limbs (the last only partially), and the high limbs stay zero.
        let cell: [M31Wrapper<Var>; MEMORY_VALUES_LIMBS] = array::from_fn(|i| {
            let start = i * LIMB_BITS;
            if start >= bits.len() {
                return zero.clone();
            }

            let chunk = &bits[start..(start + LIMB_BITS).min(bits.len())];
            let limb = Simd::combine_bits(context, chunk);
            M31Wrapper::new_unsafe(Simd::unpack_idx(context, &limb, 0))
        });
        outputs.push(cell);
    }
    debug_assert_eq!(outputs.len(), N_OUTPUTS);
    outputs
}

impl<Value: IValue> CairoStatement<Value> {
    /// Constructs a new Cairo statement from the serialized auxiliary data, outputs, program,
    /// enabled bits, preprocessed root and trace variant.
    ///
    /// See AuxData::parse_from_vars for the layout of `serialized_aux_data`.
    ///
    /// The digest words are guessed from `output_hash` and become the circuit's public output.
    /// They are split back into 9-bit limbs (see `output_limbs_from_hash`) for the public logup
    /// sum, which binds them to the proven public memory.
    ///
    /// The active components are derived from `enabled_bits` (which has one flag per component in
    /// the full list returned by `all_components()`).
    pub fn new(
        context: &mut Context<Value>,
        serialized_aux_data: Vec<M31>,
        output_hash: HashValue<Value>,
        program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
        enabled_bits: Vec<bool>,
        preprocessed_root: HashValue<QM31>,
        preprocessed_trace_variant: PreProcessedTraceVariant,
    ) -> Self {
        let components = enabled_components::<Value>(&enabled_bits);
        let n_components = components.len();
        let output_hash = output_hash.guess(context);
        context.set_outputs(&output_hash.iter().map(|word| *word.get()).collect_vec());

        let outputs = output_limbs_from_hash(context, &output_hash);

        let aux_data_len = AUX_DATA_FIXED_LEN + program.len() + n_components;
        assert_eq!(serialized_aux_data.len(), aux_data_len);

        let aux_data_vars: Vec<Var> = serialized_aux_data
            .iter()
            .map(|&m31| *M31Wrapper::new_unsafe(Value::from_qm31(m31.into())).guess(context).get())
            .collect_vec();

        let aux_data = AuxData::parse_from_vars(&aux_data_vars, program.len(), n_components);
        let packed_component_log_sizes = Simd::pack(context, &aux_data.component_log_sizes[..]);

        Self {
            aux_data,
            packed_component_log_sizes,
            program,
            outputs,
            components,
            enabled_bits,
            preprocessed_root,
            preprocessed_trace_variant,
        }
    }
}

impl<Value: IValue> Statement<Value> for CairoStatement<Value> {
    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
        &self.components
    }

    fn get_component_log_sizes(&self) -> &Simd {
        &self.packed_component_log_sizes
    }

    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<U32Wrapper<Var>>> {
        let Self {
            components: _components,
            enabled_bits,
            aux_data,
            program,
            outputs,
            preprocessed_root: _preprocessed_root,
            packed_component_log_sizes: _packed_component_log_sizes,
            preprocessed_trace_variant: _preprocessed_trace_variant,
        } = self;

        // Converts a list of M31 vars into u32 words (one word per var), zero-padded to a multiple
        // of four words. The padding mirrors the QM31-packed layout the channel expects (each QM31
        // holds four words; partial chunks pad with zeros), so the mixed words match what the proof
        // was generated with. Each returned list is mixed into the channel as one `mix_u32s` call.
        let to_padded_u32_words = |ctx: &mut Context<Value>, vars: Vec<Var>| {
            let mut words: Vec<U32Wrapper<Var>> =
                vars.into_iter().map(|v| U32Wrapper::new_unsafe(m31_to_u32(ctx, v))).collect();
            let pad = (4 - words.len() % 4) % 4;
            for _ in 0..pad {
                words.push(U32Wrapper::new_unsafe(ctx.zero()));
            }
            words
        };

        // Mix the (hardcoded) enable bits into the channel for compatibility with the Cairo1
        // verifier: the count (in its own group), then one word per bit.
        let enable_count = context.constant((enabled_bits.len() as u32).into());
        let enable_count_words = to_padded_u32_words(context, vec![enable_count]);
        let enable_bit_vars =
            enabled_bits.iter().map(|&b| context.constant(u32::from(b).into())).collect_vec();
        let enable_bits_words = to_padded_u32_words(context, enable_bit_vars);

        // Component log sizes, one word per size.
        let log_size_vars = aux_data.component_log_sizes.iter().map(|v| *v.get()).collect_vec();
        let log_sizes_words = to_padded_u32_words(context, log_size_vars);

        // Program length (in its own group), then the aux data fields in
        // `AuxData::parse_from_vars` order.
        let program_len = context.constant((program.len() as u32).into());
        let program_len_words = to_padded_u32_words(context, vec![program_len]);
        let aux_data_vars = chain!(
            [
                aux_data.initial_state.pc,
                aux_data.initial_state.ap,
                aux_data.initial_state.fp,
                aux_data.final_state.pc,
                aux_data.final_state.ap,
                aux_data.final_state.fp
            ],
            aux_data
                .segment_ranges
                .iter()
                .flat_map(|r| { [r.start.id, r.start.value, r.end.id, r.end.value] }),
            aux_data.safe_call_ids.iter().copied(),
            aux_data.output_ids.iter().copied(),
            aux_data.program_ids.iter().copied(),
        )
        .collect_vec();
        let aux_data_words = to_padded_u32_words(context, aux_data_vars);

        // Mix the outputs as 9-bit limbs into the channel.
        let output_limb_vars = outputs.iter().flatten().map(|limb| *limb.get()).collect_vec();
        let n_output_bytes = 4 * output_limb_vars.len();
        let output_limb_words = to_padded_u32_words(context, output_limb_vars);
        let output_hash = blake2s_u32s(context, output_limb_words, n_output_bytes);

        // Compute the program hash at circuit construction time.
        let flat_program = pack_into_qm31s(program.iter().flatten().cloned());
        let program_hash = IValue::blake2s(&flat_program, flat_program.len() * 16);
        let program_hash_words = program_hash
            .0
            .iter()
            .map(|word| U32Wrapper::new_unsafe(context.constant(*word.get())))
            .collect_vec();

        vec![
            enable_count_words,
            enable_bits_words,
            log_sizes_words,
            program_len_words,
            aux_data_words,
            output_hash.to_vec(),
            program_hash_words,
        ]
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let program_as_constants = self
            .program
            .iter()
            .map(|value_limbs| {
                value_limbs.map(|limb| M31Wrapper::new_unsafe(context.constant(limb.into())))
            })
            .collect_vec();

        public_logup_sum(
            context,
            &self.aux_data,
            &program_as_constants,
            &self.outputs,
            interaction_elements,
        )
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_trace_variant.to_preprocessed_trace().ids()
    }

    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
        let segment_ranges = &self.aux_data.segment_ranges;
        let public_params: HashMap<String, Var> = HashMap::from_iter(
            [
                ("output_segment_start", &segment_ranges[0]),
                ("pedersen_builtin_segment_start", &segment_ranges[1]),
                ("range_check_builtin_segment_start", &segment_ranges[2]),
                ("ecdsa_builtin_segment_start", &segment_ranges[3]),
                ("bitwise_builtin_segment_start", &segment_ranges[4]),
                ("ec_op_builtin_segment_start", &segment_ranges[5]),
                ("keccak_builtin_segment_start", &segment_ranges[6]),
                ("poseidon_builtin_segment_start", &segment_ranges[7]),
                ("range_check96_builtin_segment_start", &segment_ranges[8]),
                ("add_mod_builtin_segment_start", &segment_ranges[9]),
                ("mul_mod_builtin_segment_start", &segment_ranges[10]),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.start.value)),
        );
        public_params
    }

    fn verify_claim(
        &self,
        context: &mut Context<Value>,
        component_sizes: &[Var],
        shifted_relation_uses: &HashMap<String, Var>,
    ) {
        let AuxData {
            initial_state,
            final_state,
            segment_ranges: _,
            safe_call_ids: _,
            output_ids: _,
            program_ids: _,
            component_log_sizes: _,
        } = &self.aux_data;

        self.verify_builtins(context, component_sizes);
        // TODO(ilya): Consider adding sanity checks on the content of the program segment.

        let CasmState { pc: initial_pc, ap: initial_ap, fp: initial_fp } = initial_state;
        let CasmState { pc: final_pc, ap: final_ap, fp: final_fp } = final_state;

        // A vector of values that are going to be range checked to 29 bits.
        let mut range_checks = vec![];

        eq(context, *initial_pc, context.one());
        // Check that initial_pc (== 1) + 2 < initial_ap.
        // i.e. 3 < initial_ap < 2**29 + 4.
        // At this point we actually know that `initial_ap < 2**29` because we enforced it when
        // computing the safe_call logup sum.
        range_checks.push(eval!(context, (*initial_ap) - (context.constant(4.into()))));

        eq(context, *initial_fp, *final_fp);
        eq(context, *initial_fp, *initial_ap);
        let expected_final_pc = context.constant(5.into());
        eq(context, *final_pc, expected_final_pc);
        // Check that the initial_ap <= final_ap.
        // i.e. 0 <= final_ap - initial_ap < 2**29.
        range_checks.push(eval!(context, (*final_ap) - (*initial_ap)));

        let rc_simd = Simd::pack(
            context,
            &range_checks.iter().map(|value| M31Wrapper::new_unsafe(*value)).collect_vec(),
        );
        extract_bits(context, &rc_simd, MEMORY_ADDRESS_BITS);

        // Sanity check: ensure that the maximum address in the address_to_id is at most 2**29.
        // Assumes that there is only one ADDRESS_TO_ID component and that it uses Seq.
        const {
            assert!(
                MEMORY_ADDRESS_TO_ID_SPLIT * (1 << MAX_SEQUENCE_LOG_SIZE)
                    <= (1 << MEMORY_ADDRESS_BITS)
            )
        };

        let shifted_opcode_relation_uses =
            Simd::from_packed(vec![shifted_relation_uses["Opcodes"]], 1);
        extract_bits(
            context,
            &shifted_opcode_relation_uses,
            MEMORY_ADDRESS_BITS - u32::try_from(RELATION_USES_NUM_ROWS_SHIFT).unwrap(),
        );
    }

    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var> {
        HashValue(std::array::from_fn(|i| {
            U32Wrapper::new_unsafe(context.constant(*self.preprocessed_root[i].get()))
        }))
    }
}

pub fn segment_ranges_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    segment_ranges: &[SegmentRange<Var>; N_SEGMENTS],
    mut argument_address: Var,
    mut return_value_address: Var,
) -> Var {
    let one = context.one();
    let mut sum = context.zero();
    for (i, segment_range) in segment_ranges.iter().enumerate() {
        if i != 0 {
            argument_address = eval!(context, (argument_address) + (one));
            return_value_address = eval!(context, (return_value_address) + (one));
        }

        let start_value_limbs = split_address_to_9bit_limbs(context, segment_range.start.value);
        let segment_start_logup_term = public_memory_logup_terms(
            context,
            interaction_elements,
            argument_address,
            segment_range.start.id,
            &start_value_limbs,
        );
        sum = eval!(context, (sum) + (segment_start_logup_term));
        let end_value_limbs = split_address_to_9bit_limbs(context, segment_range.end.value);
        let segment_end_logup_term = public_memory_logup_terms(
            context,
            interaction_elements,
            return_value_address,
            segment_range.end.id,
            &end_value_limbs,
        );
        sum = eval!(context, (sum) + (segment_end_logup_term));
    }

    sum
}

/// Computes the address to id and id to value logup terms for a public memory value, returning the
/// sum.
fn public_memory_logup_terms<'a>(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    address: Var,
    id: Var,
    value_limbs: impl IntoIterator<Item = &'a Var>,
) -> Var {
    let memory_address_to_id_relation_id =
        context.constant(MEMORY_ADDRESS_TO_ID_RELATION_ID.into());
    let address_to_id_logup_term = logup_use_term(
        context,
        &[memory_address_to_id_relation_id, address, id],
        interaction_elements,
    );

    let memory_id_to_big_relation_id = context.constant(MEMORY_ID_TO_BIG_RELATION_ID.into());
    let elements =
        chain!([memory_id_to_big_relation_id, id], value_limbs.into_iter().cloned()).collect_vec();
    let id_to_value_logup_term = logup_use_term(context, &elements, interaction_elements);
    eval!(context, (address_to_id_logup_term) + (id_to_value_logup_term))
}

pub fn memory_segment_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    start_address: Var,
    ids: &[Var],
    memory_values: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
) -> Var {
    let one = context.one();
    let mut sum = context.zero();

    let mut address = start_address;
    for (i, (&id, value_limbs)) in zip_eq(ids, memory_values).enumerate() {
        if i != 0 {
            address = eval!(context, (address) + (one));
        }

        let logup_term = public_memory_logup_terms(
            context,
            interaction_elements,
            address,
            id,
            value_limbs.iter().map(|limb| limb.get()),
        );
        sum = eval!(context, (sum) + (logup_term));
    }

    sum
}

pub fn public_logup_sum(
    context: &mut Context<impl IValue>,
    aux_data: &AuxData,
    program: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
    outputs: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
    interaction_elements: [Var; 2],
) -> Var {
    let AuxData {
        initial_state,
        final_state,
        segment_ranges,
        safe_call_ids,
        output_ids,
        program_ids,
        component_log_sizes: _,
    } = aux_data;
    let initial_ap = initial_state.ap;
    let final_ap = final_state.ap;
    let final_state_logup_term = aux_data.final_state.logup_term(context, interaction_elements);
    let initial_state_logup_term = aux_data.initial_state.logup_term(context, interaction_elements);
    let mut sum = eval!(context, (final_state_logup_term) - (initial_state_logup_term));

    let one = context.one();
    let safe_call_addresses = vec![
        eval!(context, (initial_ap) - (context.constant(QM31::from(2)))),
        eval!(context, (initial_ap) - (one)),
    ];

    // Enforce correct initialization of the safe call memory section:
    // memory[initial_ap - 2] = (safe_call_id0, initial_ap)
    // memory[initial_ap - 1] = (safe_call_id1, 0).
    let split_initial_ap = split_address_to_9bit_limbs(context, initial_ap);
    // The value of memory[initial_ap - 1] is 0, so its 9-bit limbs are all zeros.
    // Passing an empty slice to id_to_big_logup_term is equivalent to passing [0, 0, 0, 0]
    // because trailing zeros don't affect the polynomial combination in combine_term.
    let safe_call_values = [split_initial_ap.as_slice(), &[]];
    for ((address, id), value_limbs) in
        zip_eq(zip_eq(safe_call_addresses, safe_call_ids), safe_call_values)
    {
        let logup_term =
            public_memory_logup_terms(context, interaction_elements, address, *id, value_limbs);
        sum = eval!(context, (sum) + (logup_term));
    }

    let argument_address = initial_ap;
    let return_value_address =
        eval!(context, (final_ap) - (context.constant(QM31::from(N_SEGMENTS as u32))));
    let segment_ranges_logup_sum = segment_ranges_logup_sum(
        context,
        interaction_elements,
        segment_ranges,
        argument_address,
        return_value_address,
    );
    sum = eval!(context, (sum) + (segment_ranges_logup_sum));

    let output_logup_sum = memory_segment_logup_sum(
        context,
        interaction_elements,
        segment_ranges[0].start.value,
        output_ids,
        outputs,
    );
    sum = eval!(context, (sum) + (output_logup_sum));

    let program_logup_sum = memory_segment_logup_sum(
        context,
        interaction_elements,
        initial_state.pc,
        program_ids,
        program,
    );
    sum = eval!(context, (sum) + (program_logup_sum));

    sum
}
