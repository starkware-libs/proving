use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::ops::{Guess, div, from_partial_evals, im, inv};
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use circuits::{EXTENSION_DEGREE, eval};
use indexmap::IndexMap;
use itertools::{Itertools, chain, zip_eq};
use num_traits::zero;
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circle::{add_points, double_point, double_x, generator_point};
use crate::proof::{Proof, ProofConfig};
use crate::select_queries::Queries;

const COMPOSITION_SPLIT: usize = 2;
pub const N_COMPOSITION_COLUMNS: usize = COMPOSITION_SPLIT * EXTENSION_DEGREE;

#[cfg(test)]
#[path = "oods_test.rs"]
pub mod test;

/// Represents the sampled values in the evaluation domain.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalDomainSamples<T> {
    /// `data[trace_idx][column_idx][query_idx]` is the `M31` value of the column `column_idx` in
    /// trace `trace_idx` at the `query_idx` query.
    data: Vec<Vec<Vec<M31Wrapper<T>>>>,
}

impl<T> EvalDomainSamples<T> {
    pub fn n_traces(&self) -> usize {
        self.data.len()
    }

    /// Returns the sampled value for the given trace, query, and column.
    pub fn at(&self, trace_idx: usize, column_idx: usize, query_idx: usize) -> &T {
        self.data[trace_idx][column_idx][query_idx].get()
    }

    /// Returns the data vector for the given trace.
    pub fn data_for_trace(&self, trace_idx: usize) -> &[Vec<M31Wrapper<T>>] {
        &self.data[trace_idx]
    }

    /// Validates that the size of the vectors in the struct are consistent with the
    /// config parameters.
    pub fn validate_structure(&self, n_columns_per_trace: &[usize], n_queries: usize) {
        for (trace_data, n_columns) in zip_eq(&self.data, n_columns_per_trace) {
            assert_eq!(trace_data.len(), *n_columns);
            for query_data in trace_data {
                assert_eq!(query_data.len(), n_queries);
            }
        }
    }
}

impl EvalDomainSamples<QM31> {
    /// Constructs a new [EvalDomainSamples] from the given data.
    pub fn from_m31s(data: Vec<Vec<Vec<M31>>>) -> Self {
        Self {
            data: data
                .iter()
                .map(|v| v.iter().map(|v| v.iter().map(|v| (*v).into()).collect()).collect())
                .collect(),
        }
    }
}

impl<Value: IValue> Guess<Value> for EvalDomainSamples<Value> {
    type Target = EvalDomainSamples<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        EvalDomainSamples { data: self.data.guess(context) }
    }
}

pub fn empty_eval_domain_samples(
    n_columns_per_trace: &[usize],
    n_queries: usize,
) -> EvalDomainSamples<NoValue> {
    EvalDomainSamples {
        data: n_columns_per_trace
            .iter()
            .map(|n_columns| vec![vec![M31Wrapper::from(NoValue); n_queries]; *n_columns])
            .collect(),
    }
}

/// Given the generator of the lifted trace and the component sizes, computes the period
/// generator for each component.
/// The period generator of a component is component_size * trace_gen.
///
/// Assumptions:
/// - All component sizes are powers of two.
pub fn period_generators(
    context: &mut Context<impl IValue>,
    trace_gen: CirclePoint<M31>,
    component_sizes_bits: &[Simd],
) -> Vec<CirclePoint<Var>> {
    let mut period_gen = CirclePoint {
        x: M31Wrapper::new_unsafe(context.constant(QM31::from_m31(
            trace_gen.x,
            zero(),
            zero(),
            zero(),
        ))),
        y: M31Wrapper::new_unsafe(context.constant(QM31::from_m31(
            trace_gen.y,
            zero(),
            zero(),
            zero(),
        ))),
    };

    let bits_0 = &component_sizes_bits[0];
    let mut res = CirclePoint {
        x: Simd::scalar_mul(context, bits_0, &period_gen.x),
        y: Simd::scalar_mul(context, bits_0, &period_gen.y),
    };

    for bit in component_sizes_bits.iter().skip(1) {
        period_gen = double_point(context, &period_gen);

        let zero_or_x = Simd::scalar_mul(context, bit, &period_gen.x);
        let zero_or_y = Simd::scalar_mul(context, bit, &period_gen.y);
        res = CirclePoint {
            x: Simd::add(context, &res.x, &zero_or_x),
            y: Simd::add(context, &res.y, &zero_or_y),
        };
    }

    zip_eq(Simd::unpack(context, &res.x), Simd::unpack(context, &res.y))
        .map(|(x, y)| CirclePoint { x, y })
        .collect()
}

/// Computes the expected value of the composition polynomial at the OODS point, based on the
/// broken composition polynomial commitment.
pub fn extract_expected_composition_eval(
    context: &mut Context<impl IValue>,
    composition_eval_at_oods: &[Var; N_COMPOSITION_COLUMNS],
    oods_point: CirclePoint<Var>,
    max_log_degree_bound: usize,
) -> Var {
    let composition_eval_at_oods_left =
        from_partial_evals(context, composition_eval_at_oods[0..4].try_into().unwrap());
    let composition_eval_at_oods_right =
        from_partial_evals(context, composition_eval_at_oods[4..8].try_into().unwrap());

    // Compute:
    //  `x = pi^{max_log_degree_bound - 2}(oods_point.x) = pi(pi(...pi(oods_point.x)...))`.
    let mut x = oods_point.x;
    for _ in 0..max_log_degree_bound - 2 {
        x = double_x(context, x);
    }

    eval!(context, (composition_eval_at_oods_left) + ((x) * (composition_eval_at_oods_right)))
}

/// An OODS response, claiming that the value of column `column_idx` in trace `trace_idx`
/// at (OODS) point `pt` is `value`.
///
/// `pt` can be either the OODS point itself or its previous point:
///   `(OODS point) - (trace generator)`.
pub struct OodsResponse {
    /// The index of the trace.
    pub trace_idx: usize,
    /// The index of the column.
    pub column_idx: usize,
    /// The point to evaluate the column at.
    pub pt: CirclePoint<Var>,
    /// The expected value of the column at `pt`.
    pub value: Var,
}

/// Computes the list of [OodsResponse]s that will be validated using FRI.
///
/// The order is consistent with the order dictated by the stwo prover.
pub fn collect_oods_responses(
    context: &mut Context<impl IValue>,
    config: &ProofConfig,
    oods_point: CirclePoint<Var>,
    component_sizes_bits: &[Simd],
    proof: &Proof<Var>,
) -> Vec<OodsResponse> {
    // Build the periodicity sample points for each column in the interaction trace.
    let trace_gen = generator_point(config.log_trace_size());
    let period_generators_per_component =
        period_generators(context, trace_gen, component_sizes_bits);
    let periodicity_sample_points_per_component = period_generators_per_component
        .into_iter()
        .map(|pt| add_points(context, &oods_point, &pt))
        .collect_vec();
    let periodicity_sample_points_per_column =
        column_periodicity_sample_points(config, &periodicity_sample_points_per_component);

    // The negation of the trace generator, as `CirclePoint<Var>`.
    let neg_trace_gen: CirclePoint<Var> = CirclePoint {
        x: context.constant(trace_gen.x.into()),
        y: context.constant((-trace_gen.y).into()),
    };
    // The point: `oods_point - neg_trace_gen`.
    let oods_point_at_prev_row = add_points(context, &oods_point, &neg_trace_gen);

    // The order below is the order dictated by the stwo prover.
    // First, we have all the `OodsResponse`s for the OODS point, then for the previous point.
    chain!(
        (0..config.n_preprocessed_columns).map(|column_idx| OodsResponse {
            trace_idx: 0,
            column_idx,
            pt: oods_point,
            value: proof.preprocessed_columns_at_oods[column_idx],
        }),
        (0..config.n_trace_columns).map(|column_idx| OodsResponse {
            trace_idx: 1,
            column_idx,
            pt: oods_point,
            value: proof.trace_at_oods[column_idx],
        }),
        zip_eq(
            config.cumulative_sum_columns.iter().enumerate(),
            periodicity_sample_points_per_column
        )
        .flat_map(|((column_idx, is_cumulative_sum), periodicity_sample_point)| {
            let at_oods_response = OodsResponse {
                trace_idx: 2,
                column_idx,
                pt: oods_point,
                value: proof.interaction_at_oods[column_idx].at_oods,
            };

            if !is_cumulative_sum {
                // This is a regular interaction column with a single sample at the OODS point.
                return vec![at_oods_response];
            };

            vec![
                // Periodicity check.
                OodsResponse {
                    trace_idx: 2,
                    column_idx,
                    pt: periodicity_sample_point,
                    value: proof.interaction_at_oods[column_idx].at_oods,
                },
                // Previous row.
                OodsResponse {
                    trace_idx: 2,
                    column_idx,
                    pt: oods_point_at_prev_row,
                    value: proof.interaction_at_oods[column_idx].at_prev.unwrap(),
                },
                // OODS point.
                at_oods_response,
            ]
        }),
        (0..N_COMPOSITION_COLUMNS).map(|column_idx| OodsResponse {
            trace_idx: 3,
            column_idx,
            pt: oods_point,
            value: proof.composition_eval_at_oods[column_idx],
        }),
    )
    .collect()
}

/// A struct for handling auxiliary values related to OODS data corresponding to a point `(px, py)`.
pub struct OodsPointAuxiliary {
    /// Coefficients `(d, e, f)` of the denominator line equation through (px, py), conj((px, py)).

    /// The `d` coefficient of the denominator line, equals `im(py)`.
    pub d: Var,
    /// The `e` coefficient of the denominator line, equals `im(px)`.
    pub e: Var,
    /// The `f` coefficient of the denominator line, equals `d * px - e * py`.
    pub f: Var,

    /// Batched versions of coefficients `(a, b, c)` of the numerator polynomial
    /// `(c_i * f_i(q) - a_i * qy - b_i) * alpha^i`. `a, b` are summed over all columns `f_i`
    /// corresponding to the point. The `c_i * alpha^i` coefficients are stored in a vector
    /// together with the trace and column index of the corresponding column. Note that for all i,
    /// c_i = im(py) = d.
    pub c_vec: Vec<(Var, usize, usize)>,
    /// `sum_{i: p_i = p} a_i * alpha^i`, where `a_i = im(v_i)`.
    pub a_sum: Var,
    /// `sum_{i: p_i = p} b_i * alpha^i`, where `b_i = c_i * v_i - a_i * py`.
    pub b_sum: Var,

    /// Private intermediate variables retained for computing `b_sum`.
    mul_v_sum: Var,
    py: Var,
}

impl OodsPointAuxiliary {
    /// Compute the (d, e, f) values matching (px, py), and initialize the accumulators to null.
    pub fn new(context: &mut Context<impl IValue>, px: Var, py: Var) -> Self {
        let d = im(context, py);
        let e = im(context, px);
        let f = eval!(context, ((d) * (px)) - ((e) * (py)));
        let c_vec = Vec::new();
        let [a_sum, b_sum, mul_v_sum] = [context.zero(); 3];

        Self { d, e, f, c_vec, a_sum, b_sum, mul_v_sum, py }
    }

    /// Add the OodsResponse `r` data to c_vec and the a, mul_v accumulators.
    pub fn accumulate(
        &mut self,
        context: &mut Context<impl IValue>,
        alpha_power: Var,
        r: &OodsResponse,
    ) {
        self.c_vec.push((eval!(context, (self.d) * (alpha_power)), r.trace_idx, r.column_idx));
        let v_im = im(context, r.value);
        self.a_sum = eval!(context, (self.a_sum) + ((alpha_power) * (v_im)));
        self.mul_v_sum = eval!(context, (self.mul_v_sum) + ((alpha_power) * (r.value)));
    }

    /// Finalize the value of `b_sum` from the accumulated a, mul_v sums.
    pub fn finalize(&mut self, context: &mut Context<impl IValue>) {
        self.b_sum = eval!(context, ((self.d) * (self.mul_v_sum)) - ((self.a_sum) * (self.py)));
    }
}

/// In order to validate the [OodsResponse]s, we use FRI to show that the following rational
/// function is in fact a polynomial:
/// ```plain
///   (-2u) * sum_i (
///       alpha^i * (a * y + b - c * column[column_idx](x, y)) / (d * x - e * y - f)
///   )
/// ```
/// where:
/// ```plain
///    a = im(value)
///    c = im(pt.y)
///    b = c * value - a * pt.y
///
///    d = im(pt.y)
///    e = im(pt.x)
///    f = d * pt.x - e * pt.y
/// ```
///
/// Note that:
/// (1) `d * x - e * y - f` vanishes on the points `pt` and `conj(pt)`.
/// (2) `(a * y + b) / c` evaluates to `value` at `pt.y` and to `conj(value)` at `conj(pt.y)`.
///
/// The function computes the inputs to FRI, which are the evaluations of the above rational
/// function at the given (evaluation domain) queries.
pub fn compute_fri_input(
    context: &mut Context<impl IValue>,
    oods_responses: &[OodsResponse],
    queries: &Queries,
    trace_queries: &EvalDomainSamples<Var>,
    alpha: Var,
) -> Vec<Var> {
    // A dict matching each (pt.x, pt.y) to its auxiliary data.
    let mut aux_dict = IndexMap::<(usize, usize), OodsPointAuxiliary>::new();

    // Multiply all the quotient coefficients by `-2u` to compensate for the different
    // denominator computation with respect to stwo.
    let mut alpha_pow = context.constant(-qm31_from_u32s(0, 0, 2, 0));

    // Initialize, accumulate and finalize the auxiliary data derived from the OodsResponses.
    for (i, r) in oods_responses.iter().enumerate() {
        if i > 0 {
            // Compute the next alpha power.
            alpha_pow = eval!(context, (alpha_pow) * (alpha));
        }
        let key = (r.pt.x.idx, r.pt.y.idx);
        // Initialize a new OodsPointAuxiliary if entry is vacant.
        let entry =
            aux_dict.entry(key).or_insert_with(|| OodsPointAuxiliary::new(context, r.pt.x, r.pt.y));
        entry.accumulate(context, alpha_pow, r);
    }
    let mut prod = context.one();
    for (i, aux) in aux_dict.values_mut().enumerate() {
        aux.finalize(context);
        if i == 0 {
            prod = aux.d;
        } else {
            prod = eval!(context, (prod) * (aux.d));
        }
    }
    // Assert d = im(r.pt.y) != 0 for all OODS points r. This assert guarantees:
    //
    // 1. Non-zero query_value coefficients: The coefficient of `query_value` in each quotient
    //    numerator is d. If d = 0 the numerator vanishes regardless of the column value, breaking
    //    the binding between the committed column values and the OODS evaluations.
    //
    // 2. Non-zero denominators: The denominator `d * x - e * y + f` vanishes on the circle only at
    //    pt and conj(pt). Query points (q.x, q.y) are M31 points with im(q.y) = 0, so d != 0
    //    ensures q is neither pt nor conj(pt), and the denominators are non-zero.
    inv(context, prod);

    let query_point_x = Simd::unpack(context, &queries.points.x);
    let query_point_y = Simd::unpack(context, &queries.points.y);

    let mut fri_queries = Vec::with_capacity(query_point_x.len());
    for (query_idx, (q_x, q_y)) in zip_eq(&query_point_x, &query_point_y).enumerate() {
        let mut sum = context.zero();

        for aux in aux_dict.values() {
            // The `a` and `b` contributions for the point were already batched.
            let mut numerator = eval!(context, ((aux.a_sum) * (*q_y)) + (aux.b_sum));

            // Subtract the `c * query_value_at_column` for each column corresponding to the point.
            for (coeff, trace_idx, column_idx) in &aux.c_vec {
                let query_value_at_column = *trace_queries.at(*trace_idx, *column_idx, query_idx);
                numerator = eval!(context, (numerator) - ((*coeff) * (query_value_at_column)));
            }

            // Compute the denominator line at (q_x, q_y).
            let denominator = eval!(context, ((aux.d) * (*q_x)) - (((aux.e) * (*q_y)) + (aux.f)));
            // Non-zero: query points are M31 circle points (im(q.y)=0), so the line
            // through an OODS sample point and its conjugate does not pass through them when d !=
            // 0.
            let quotient = div(context, numerator, denominator);
            sum = eval!(context, (sum) + (quotient));
        }

        fri_queries.push(sum);
    }

    fri_queries
}

/// Given the periodicity sample points for each component, returns the periodicity sample points
/// for each column in the interaction trace.
/// The periodicity sample points are the sample points used for the periodicity check.
fn column_periodicity_sample_points(
    config: &ProofConfig,
    sample_points_per_component: &[CirclePoint<Var>],
) -> Vec<CirclePoint<Var>> {
    let mut periodicity_sample_points_per_column = Vec::with_capacity(config.n_interaction_columns);
    for (component_shape, sample_point) in
        zip_eq(&config.component_shapes, sample_points_per_component)
    {
        periodicity_sample_points_per_column
            .extend(vec![sample_point; component_shape.interaction_columns]);
    }
    periodicity_sample_points_per_column
}
