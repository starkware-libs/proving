use std::collections::HashMap;

use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::{from_partial_evals, inv, mul};
use circuits::simd::Simd;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::circle::denom_inverse;
use crate::logup::{LogupTerm, logup_term, pair_logup_constraint, single_logup_constraint};
use crate::order_hash_map::OrderedHashMap;
use crate::proof::{InteractionAtOods, ProofConfig};
use crate::statement::{EvaluateArgs, Statement};

#[derive(Debug)]
pub struct RelationUse {
    pub relation_id: &'static str,
    pub uses: u64,
}

// Data associated with a specific component.
pub struct ComponentData<'a> {
    /// The trace columns of the component.
    pub trace_columns: &'a [Var],
    /// The interaction columns of the component.
    pub interaction_columns: &'a [InteractionAtOods<Var>],
    /// The number of instances in the component.
    pub n_instances: Var,

    /// The index of the component.
    index: usize,

    /// Simd of bits representing the `n_instances` in the component.
    n_instances_bits: &'a [Simd],
}

pub trait ComponentDataTrait<Value: IValue> {
    fn trace_columns(&self) -> &[Var];

    fn interaction_columns(&self) -> &[InteractionAtOods<Var>];

    fn n_instances(&self) -> Var;

    /// Returns one of the bits of the component number of rows (bit 0 is LSB).
    /// Because the number of rows is always a power of two, only one of the bits
    /// will be 1 and the rest will be zero.
    fn get_n_instances_bit(&self, context: &mut Context<Value>, bit: usize) -> Var;

    // The number of bits required to represent the size of the largest supported component size.
    // Note that this is one more than log2(max_component_size), because 2**n is a (n+1)-bit
    // number.
    fn max_component_size_bits(&self) -> usize;
}

impl<'a, Value: IValue> ComponentDataTrait<Value> for ComponentData<'a> {
    fn trace_columns(&self) -> &[Var] {
        self.trace_columns
    }

    fn interaction_columns(&self) -> &[InteractionAtOods<Var>] {
        self.interaction_columns
    }

    fn n_instances(&self) -> Var {
        self.n_instances
    }

    fn get_n_instances_bit(&self, context: &mut Context<Value>, bit: usize) -> Var {
        Simd::unpack_idx(context, &self.n_instances_bits[bit], self.index)
    }

    fn max_component_size_bits(&self) -> usize {
        self.n_instances_bits.len()
    }
}

/// Accumulates a pseudo-random linear combination of constraint evaluations at the OODS point and
/// the previous point.
///
/// Maintains the running value:
///   accumulation <- accumulation * composition_polynomial_coeff + c_i
/// so that after N constraints:
///   accumulation = Σ_{i=0..N-1} composition_polynomial_coeff^{N-1-i} * c_i.
pub struct CompositionConstraintAccumulator {
    /// The OODS samples for the preprocessed columns.
    /// Each component will consume a subset of these samples.
    pub preprocessed_columns: HashMap<PreProcessedColumnId, Var>,
    pub public_params: HashMap<String, Var>,
    /// The random coefficient for the composition polynomial.
    pub composition_polynomial_coeff: Var,
    /// The interaction elements for the logup sums constraint.
    pub interaction_elements: [Var; 2],
    /// Running accumulator over constraint evaluations at the OODS point and the previous point.
    pub accumulation: Var,
    pub terms: Vec<LogupTerm>,
}

impl CompositionConstraintAccumulator {
    pub fn new(
        context: &mut Context<impl IValue>,
        preprocessed_columns: HashMap<PreProcessedColumnId, Var>,
        public_params: HashMap<String, Var>,
        composition_polynomial_coeff: Var,
        interaction_elements: [Var; 2],
    ) -> Self {
        Self {
            preprocessed_columns,
            public_params,
            composition_polynomial_coeff,
            interaction_elements,
            accumulation: context.zero(),
            terms: Vec::new(),
        }
    }

    /// Incorporate the next constraint evaluation at the OODS point.
    pub fn accumulate(&mut self, context: &mut Context<impl IValue>, constraint_eval_at_oods: Var) {
        let shifted_accumulation =
            eval!(context, (self.accumulation) * (self.composition_polynomial_coeff));
        self.accumulation = eval!(context, (shifted_accumulation) + (constraint_eval_at_oods));
    }

    /// Finish accumulation and return the combined value.
    ///
    /// Panics if not all expected samples/claimed sums have been consumed.
    pub fn finalize(self) -> Var {
        self.accumulation
    }

    pub fn get_preprocessed_column(&self, preprocessed_column_idx: &PreProcessedColumnId) -> Var {
        *self.preprocessed_columns.get(preprocessed_column_idx).unwrap()
    }

    pub fn add_constraint(
        &mut self,
        context: &mut Context<impl IValue>,
        constraint_eval_at_oods: Var,
    ) {
        self.accumulate(context, constraint_eval_at_oods);
    }

    pub fn add_to_relation(
        &mut self,
        context: &mut Context<impl IValue>,
        numerator: Var,
        element: &[Var],
    ) {
        self.terms.push(logup_term(context, self.interaction_elements, numerator, element));
    }

    pub fn finalize_logup_in_pairs<Value: IValue>(
        &mut self,
        context: &mut Context<Value>,
        interaction_columns: &[InteractionAtOods<Var>],
        component_data: &dyn ComponentDataTrait<Value>,
        claimed_sum: Var,
    ) {
        // TODO(Gali): Get the terms from the component instead of storing them in the accumulator.
        let n_batches = self.terms.len().div_ceil(2);
        assert_eq!(interaction_columns.len(), n_batches * SECURE_EXTENSION_DEGREE);
        let (interaction_columns, last_chunk) = interaction_columns.split_last_chunk().unwrap();
        let mut prev_col_cumsum = context.zero();

        for (i, mut chunk_iter) in
            interaction_columns.iter().chunks(SECURE_EXTENSION_DEGREE).into_iter().enumerate()
        {
            let cur_cumsum = from_partial_evals(
                context,
                std::array::from_fn(|_| {
                    let interaction = chunk_iter.next().unwrap();
                    assert!(interaction.at_prev.is_none());
                    interaction.at_oods
                }),
            );
            // All pairs except the last are cumulatively summed in new interaction columns.
            let diff =
                if i > 0 { eval!(context, (cur_cumsum) - (prev_col_cumsum)) } else { cur_cumsum };
            prev_col_cumsum = cur_cumsum;

            let logup_constraint_val =
                pair_logup_constraint(context, self.terms[2 * i], self.terms[2 * i + 1], diff);
            self.add_constraint(context, logup_constraint_val);
        }

        let prev_row_cumsum =
            from_partial_evals(context, last_chunk.each_ref().map(|x| x.at_prev.unwrap()));
        let cur_cumsum = from_partial_evals(context, last_chunk.each_ref().map(|x| x.at_oods));

        let diff = eval!(context, ((cur_cumsum) - (prev_row_cumsum)) - (prev_col_cumsum));
        // n_instances = 2^log_size, which is always non-zero.
        let n_instances_inv = inv(context, component_data.n_instances());
        let cumsum_shift = mul(context, claimed_sum, n_instances_inv);
        // Instead of checking diff = num / denom, check diff = num / denom - cumsum_shift.
        // This makes (num / denom - cumsum_shift) have sum zero, which makes the constraint
        // uniform - apply on all rows.
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));

        let logup_constraint_val = if self.terms.len().is_multiple_of(2) {
            pair_logup_constraint(
                context,
                self.terms[2 * (n_batches - 1)],
                self.terms[2 * (n_batches - 1) + 1],
                shifted_diff,
            )
        } else {
            single_logup_constraint(context, self.terms[2 * (n_batches - 1)], shifted_diff)
        };
        self.add_constraint(context, logup_constraint_val);

        self.terms.clear();
    }
}

/// A trait for evaluating at some point or row.
pub trait CircuitEval<Value: IValue> {
    fn name(&self) -> String;

    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    );

    /// Returns the number of trace columns used by the component.
    fn trace_columns(&self) -> usize;

    /// Returns the number of interaction columns used by the component.
    fn interaction_columns(&self) -> usize;

    fn relation_uses_per_row(&self) -> &[RelationUse];

    /// Returns the log size of the component, or `None` if the component's log size is dynamic.
    fn log_size(
        &self,
        preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32>;
}

pub fn get_n_columns<'a, T>(columns: &mut &'a [T], n: usize) -> &'a [T] {
    if let Some(vec) = columns.split_off(..n) { vec } else { panic!("Expected {n} columns") }
}

pub fn compute_composition_polynomial<Value: IValue>(
    context: &mut Context<Value>,
    config: &ProofConfig,
    statement: &impl Statement<Value>,
    args: EvaluateArgs<'_>,
) -> Var {
    let EvaluateArgs {
        mut oods_samples,
        pt,
        log_domain_size,
        composition_polynomial_coeff,
        interaction_elements,
        claimed_sums,
        component_sizes,
        n_instances_bits,
    } = args;

    let preprocessed_columns = HashMap::from_iter(zip_eq(
        statement.get_preprocessed_column_ids(),
        oods_samples.preprocessed_columns.iter().cloned(),
    ));
    let public_params = statement.public_params(context);
    let mut evaluation_accumulator = CompositionConstraintAccumulator::new(
        context,
        preprocessed_columns,
        public_params,
        composition_polynomial_coeff,
        interaction_elements,
    );

    for (component_index, (((component, component_shape), &claimed_sum), &component_size)) in
        zip_eq(
            zip_eq(
                zip_eq(statement.get_components().values(), &config.component_shapes),
                claimed_sums,
            ),
            component_sizes,
        )
        .enumerate()
    {
        let trace_columns = get_n_columns(&mut oods_samples.trace, component_shape.trace_columns);
        let interaction_columns =
            get_n_columns(&mut oods_samples.interaction, component_shape.interaction_columns);

        let component_data = ComponentData {
            trace_columns,
            interaction_columns,
            n_instances: component_size,
            index: component_index,
            n_instances_bits,
        };

        component.evaluate(context, &component_data, &mut evaluation_accumulator);

        evaluation_accumulator.finalize_logup_in_pairs(
            context,
            interaction_columns,
            &component_data,
            claimed_sum,
        );
    }

    assert!(oods_samples.trace.is_empty(), "unconsumed trace columns");
    assert!(oods_samples.interaction.is_empty(), "unconsumed interaction columns");

    let final_evaluation = evaluation_accumulator.finalize();
    // pt is the OODS point, outside the trace coset.
    let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
    eval!(context, (final_evaluation) * (denom_inverse))
}
