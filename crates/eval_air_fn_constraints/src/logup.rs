use num_traits::Zero;
use stwo_cairo_common::prover_types::cpu::QM31;

use crate::EvaluatedLookupTerm;
use crate::assignment::*;

#[derive(Debug)]
struct LogupTerm {
    pub numerator: QM31,
    pub denominator: QM31,
}

pub fn evaluate_logup_constraints(
    assignment: &Assignment,
    lookup_terms: &[EvaluatedLookupTerm],
) -> Vec<QM31> {
    let logup_terms = build_logup_terms(assignment, lookup_terms);
    let mut result = vec![];

    // Create constraints for summing the logup terms. Every two consecutive terms are
    // summed using a single degree-3 constraint.
    for (batch_idx, logup_term_batch) in logup_terms.chunks(2).enumerate() {
        let mut prev_sum = QM31::zero();
        if batch_idx > 0 {
            prev_sum += assignment.interaction_trace[batch_idx - 1];
        }
        if batch_idx == assignment.interaction_trace.len() - 1 {
            prev_sum += assignment.last_row_sum;
            prev_sum -= assignment.claimed_sum / QM31::from(1 << assignment.log_height);
        }

        let cur_sum = assignment.interaction_trace[batch_idx];

        // evaluate the constraint cur_sum = prev_sum + sum_{term in batch}(term.numerator /
        // term.denominator)
        let eval = match logup_term_batch {
            [term] => (cur_sum - prev_sum) * term.denominator - term.numerator,
            [term1, term2] => {
                (cur_sum - prev_sum) * term1.denominator * term2.denominator
                    - term1.numerator * term2.denominator
                    - term2.numerator * term1.denominator
            }
            _ => panic!("Unexpected logup batch size"),
        };
        result.push(eval)
    }
    result
}

/// Given the list of lookups performed by a component, compute the terms that have
/// to be added to the logup sum. Each term is a fraction with:
///     - Numerator: +1 for use, -1 for yield, possibly multiplied by an enabler / multiplicity
///       value
///     - Denominator: z + sum_i(alpha^i * v_i) where z, alpha are the interaction elements for the
///       relation we look into, and v_i are the felts in the tuple we look for.
fn build_logup_terms(
    assignment: &Assignment,
    lookup_terms: &[EvaluatedLookupTerm],
) -> Vec<LogupTerm> {
    lookup_terms
        .iter()
        .map(|lookup_term| {
            let abs_numerator = lookup_term.multiplicity_value;
            let numerator = abs_numerator * lookup_term.use_or_yield_sign;
            let denominator = assignment
                .common_lookup_elements
                .compute_logup_denominator(&lookup_term.felt_values);
            LogupTerm { numerator, denominator }
        })
        .collect::<Vec<_>>()
}
