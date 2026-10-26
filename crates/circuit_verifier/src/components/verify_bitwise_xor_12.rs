use super::prelude::*;

pub const ELEM_BITS: u32 = 12;
pub const EXPAND_BITS: u32 = 2;
pub const LIMB_BITS: u32 = ELEM_BITS - EXPAND_BITS;
pub const LOG_SIZE: u32 = (ELEM_BITS - EXPAND_BITS) * 2;
pub const N_MULT_COLUMNS: usize = 1 << (EXPAND_BITS * 2);
pub const N_TRACE_COLUMNS: usize = N_MULT_COLUMNS;
pub const N_INTERACTION_COLUMNS: usize = SECURE_EXTENSION_DEGREE * N_MULT_COLUMNS.div_ceil(2);

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "verify_bitwise_xor_12".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        let relation_id = context.constant(M31::from(648362599).into());

        let a_low = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_0".to_owned() });
        let b_low = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_1".to_owned() });
        let c_low = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_2".to_owned() });

        let mut trace_cols = component_data.trace_columns().iter();
        for i in 0..1 << EXPAND_BITS {
            for j in 0..1 << EXPAND_BITS {
                let multiplicity = *trace_cols.next().unwrap();

                let a = eval!(
                    context,
                    (a_low) + (context.constant(M31::from((i << LIMB_BITS) as u32).into()))
                );
                let b = eval!(
                    context,
                    (b_low) + (context.constant(M31::from((j << LIMB_BITS) as u32).into()))
                );
                let c = eval!(
                    context,
                    (c_low) + (context.constant(M31::from(((i ^ j) << LIMB_BITS) as u32).into()))
                );

                let neg_multiplicity = eval!(context, (context.zero()) - (multiplicity));
                acc.add_to_relation(context, neg_multiplicity, &[relation_id, a, b, c]);
            }
        }
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }

    fn log_size(
        &self,
        _preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        Some(LOG_SIZE)
    }
}
