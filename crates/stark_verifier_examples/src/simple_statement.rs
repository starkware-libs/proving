use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::{Guess, inv, mul};
use circuits::simd::Simd;
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::constraint_eval::{
    CircuitEval, ComponentDataTrait, CompositionConstraintAccumulator, RelationUse,
};
use circuits_stark_verifier::logup::combine_term;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::statement::Statement;
use indexmap::IndexMap;
use num_traits::One;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use super::simple_air::FIB_SEQUENCE_LENGTH;
use crate::simple_air::{FIB_PREPROCESSED_COLUMNS, LOG_SIZE_LONG, LOG_SIZE_SHORT};

/// Log sizes of the components in [`SimpleStatement`].
pub const COMPONENT_LOG_SIZES: [u32; 2] = [LOG_SIZE_LONG, LOG_SIZE_SHORT];
pub const PREPROCESSED_COLUMN_LOG_SIZES: [u32; 2] = [LOG_SIZE_SHORT, LOG_SIZE_LONG];

pub struct SimpleStatement<Value: IValue> {
    components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    component_log_sizes: Simd,
}

pub fn simple_statement_components<Value: IValue>()
-> IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
    IndexMap::from([
        (
            "squared_fibonacci_long",
            Box::new(SquaredFibonacciComponent {
                preprocessed_column_id: PreProcessedColumnId { id: "row_const_long".to_string() },
            }) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "squared_fibonacci_short",
            Box::new(SquaredFibonacciComponent {
                preprocessed_column_id: PreProcessedColumnId { id: "row_const_short".to_string() },
            }) as Box<dyn CircuitEval<Value>>,
        ),
    ])
}

impl<Value: IValue> SimpleStatement<Value> {
    pub fn new(context: &mut Context<Value>) -> Self {
        let n_components = COMPONENT_LOG_SIZES.len();
        let packed_log_sizes = pack_into_qm31s(COMPONENT_LOG_SIZES.iter().cloned())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect::<Vec<_>>();
        let component_log_sizes = Simd::from_packed(packed_log_sizes, n_components);

        Self { components: simple_statement_components(), component_log_sizes }
    }
}

pub struct SquaredFibonacciComponent {
    pub preprocessed_column_id: PreProcessedColumnId,
}
impl<Value: IValue> CircuitEval<Value> for SquaredFibonacciComponent {
    fn name(&self) -> String {
        "squared_fibonacci".to_string()
    }

    fn trace_columns(&self) -> usize {
        4
    }

    fn interaction_columns(&self) -> usize {
        8
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[RelationUse { relation_id: "fib_relation", uses: 3 }]
    }

    fn log_size(
        &self,
        preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        preprocessed_column_log_sizes.get(&self.preprocessed_column_id).cloned()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        let const_val = acc.get_preprocessed_column(&self.preprocessed_column_id);
        let [a, b, c, d] = *component_data.trace_columns() else {
            panic!("Expected 4 trace columns")
        };

        // Constraints.
        let constraint0_val = eval!(context, (c) - ((((a) * (a)) + ((b) * (b))) + (const_val)));
        acc.add_constraint(context, constraint0_val);

        let constraint1_val = eval!(context, (d) - ((((b) * (b)) + ((c) * (c))) + (const_val)));
        acc.add_constraint(context, constraint1_val);

        // Logup constraint.
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
    }
}

fn squared_fibonacci_public_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    log_n_instances: u32,
) -> Var {
    let mut sum = context.zero();
    for j in 0..(1 << log_n_instances) {
        let mut a: M31 = M31::one();
        let mut b: M31 = j.into();
        for _ in 0..(FIB_SEQUENCE_LENGTH - 2) {
            (a, b) = (b, a * a + b * b + M31::from(j));
        }
        let elements = [context.constant(a.into()), context.constant(b.into())];
        // denom1 is evaluated at the random interaction elements, so it is non-zero with high
        // probability (denom = denom1^2 is non-zero for the same reason).
        let denom1 = combine_term(context, &elements, interaction_elements);

        let denom = eval!(context, (denom1) * (denom1));
        let numerator = eval!(context, (denom1) + (denom1));

        let denom_inv = inv(context, denom);
        let frac0 = mul(context, numerator, denom_inv);
        let frac1 = inv(context, denom1);
        let frac = eval!(context, (frac0) + (frac1));

        // Note that the sum is negated because we want to use the values that are yielded in
        // the witness.
        sum = eval!(context, (sum) - (frac));
    }
    sum
}

impl<Value: IValue> Statement<Value> for SimpleStatement<Value> {
    fn claims_to_mix(&self, _context: &mut Context<Value>) -> Vec<Vec<U32Wrapper<Var>>> {
        // The public claim is empty (mixed as an empty call).
        vec![vec![]]
    }

    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
        &self.components
    }

    fn get_component_log_sizes(&self) -> &Simd {
        &self.component_log_sizes
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();

        for log_n_instances in COMPONENT_LOG_SIZES {
            let fib_logup_sum =
                squared_fibonacci_public_logup_sum(context, interaction_elements, log_n_instances);
            sum = eval!(context, (sum) + (fib_logup_sum));
        }
        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        FIB_PREPROCESSED_COLUMNS
            .iter()
            .map(|id| PreProcessedColumnId { id: id.to_string() })
            .collect()
    }

    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var> {
        // Full (unreduced) preprocessed trace root, one 32-bit word per limb.
        let root: HashValue<QM31> = [
            1715533920, 602108671, 544739379, 1352385240, 2796078938, 3611037495, 1627440248,
            306213366,
        ]
        .into();
        HashValue(std::array::from_fn(|i| {
            U32Wrapper::new_unsafe(context.constant(*root.0[i].get()))
        }))
    }
}
