use stwo::core::fields::m31::P as M31_P;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::MAX_SEQUENCE_LOG_SIZE;

use crate::components::prelude::*;

const LARGE_MEMORY_VALUE_ID_BASE: u32 = 0x40000000; // 2^30.
const ID_TO_BIG_MAX_ROWS: u32 = 1 << MAX_SEQUENCE_LOG_SIZE;
pub const N_TRACE_COLUMNS: usize = 29;
pub const N_INTERACTION_COLUMNS: usize = 32;

pub const RELATION_USES_PER_ROW: [RelationUse; 8] = [
    RelationUse { relation_id: "RangeCheck_9_9", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_E", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_F", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9_G", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_H", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
    index: u32,
) {
    let [
        multiplicity_0_col0,
        memory_id_to_big_output_col1,
        memory_id_to_big_output_col2,
        memory_id_to_big_output_col3,
        memory_id_to_big_output_col4,
        memory_id_to_big_output_col5,
        memory_id_to_big_output_col6,
        memory_id_to_big_output_col7,
        memory_id_to_big_output_col8,
        memory_id_to_big_output_col9,
        memory_id_to_big_output_col10,
        memory_id_to_big_output_col11,
        memory_id_to_big_output_col12,
        memory_id_to_big_output_col13,
        memory_id_to_big_output_col14,
        memory_id_to_big_output_col15,
        memory_id_to_big_output_col16,
        memory_id_to_big_output_col17,
        memory_id_to_big_output_col18,
        memory_id_to_big_output_col19,
        memory_id_to_big_output_col20,
        memory_id_to_big_output_col21,
        memory_id_to_big_output_col22,
        memory_id_to_big_output_col23,
        memory_id_to_big_output_col24,
        memory_id_to_big_output_col25,
        memory_id_to_big_output_col26,
        memory_id_to_big_output_col27,
        memory_id_to_big_output_col28,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);

    range_check_mem_value_n_28::accumulate_constraints(
        &[
            eval!(context, memory_id_to_big_output_col1),
            eval!(context, memory_id_to_big_output_col2),
            eval!(context, memory_id_to_big_output_col3),
            eval!(context, memory_id_to_big_output_col4),
            eval!(context, memory_id_to_big_output_col5),
            eval!(context, memory_id_to_big_output_col6),
            eval!(context, memory_id_to_big_output_col7),
            eval!(context, memory_id_to_big_output_col8),
            eval!(context, memory_id_to_big_output_col9),
            eval!(context, memory_id_to_big_output_col10),
            eval!(context, memory_id_to_big_output_col11),
            eval!(context, memory_id_to_big_output_col12),
            eval!(context, memory_id_to_big_output_col13),
            eval!(context, memory_id_to_big_output_col14),
            eval!(context, memory_id_to_big_output_col15),
            eval!(context, memory_id_to_big_output_col16),
            eval!(context, memory_id_to_big_output_col17),
            eval!(context, memory_id_to_big_output_col18),
            eval!(context, memory_id_to_big_output_col19),
            eval!(context, memory_id_to_big_output_col20),
            eval!(context, memory_id_to_big_output_col21),
            eval!(context, memory_id_to_big_output_col22),
            eval!(context, memory_id_to_big_output_col23),
            eval!(context, memory_id_to_big_output_col24),
            eval!(context, memory_id_to_big_output_col25),
            eval!(context, memory_id_to_big_output_col26),
            eval!(context, memory_id_to_big_output_col27),
            eval!(context, memory_id_to_big_output_col28),
            eval!(context, 1),
        ],
        context,
        component_data,
        acc,
    );

    // Yield MemoryIdToBig.
    let offset = LARGE_MEMORY_VALUE_ID_BASE + index * ID_TO_BIG_MAX_ROWS;
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, (seq) + (context.constant(offset.into()))),
        eval!(context, memory_id_to_big_output_col1),
        eval!(context, memory_id_to_big_output_col2),
        eval!(context, memory_id_to_big_output_col3),
        eval!(context, memory_id_to_big_output_col4),
        eval!(context, memory_id_to_big_output_col5),
        eval!(context, memory_id_to_big_output_col6),
        eval!(context, memory_id_to_big_output_col7),
        eval!(context, memory_id_to_big_output_col8),
        eval!(context, memory_id_to_big_output_col9),
        eval!(context, memory_id_to_big_output_col10),
        eval!(context, memory_id_to_big_output_col11),
        eval!(context, memory_id_to_big_output_col12),
        eval!(context, memory_id_to_big_output_col13),
        eval!(context, memory_id_to_big_output_col14),
        eval!(context, memory_id_to_big_output_col15),
        eval!(context, memory_id_to_big_output_col16),
        eval!(context, memory_id_to_big_output_col17),
        eval!(context, memory_id_to_big_output_col18),
        eval!(context, memory_id_to_big_output_col19),
        eval!(context, memory_id_to_big_output_col20),
        eval!(context, memory_id_to_big_output_col21),
        eval!(context, memory_id_to_big_output_col22),
        eval!(context, memory_id_to_big_output_col23),
        eval!(context, memory_id_to_big_output_col24),
        eval!(context, memory_id_to_big_output_col25),
        eval!(context, memory_id_to_big_output_col26),
        eval!(context, memory_id_to_big_output_col27),
        eval!(context, memory_id_to_big_output_col28),
    ];
    let numerator_1 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_1, tuple_1);
}

pub struct Component {
    // The trace can contain multiple memory_id_to_big components, each responsible
    // for a range of IDs. The index differentiates between them and sets the ID range
    // for each.
    pub index: u32,
}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        if self.index == 0 {
            "memory_id_to_big".to_string()
        } else {
            format!("memory_id_to_big_{}", self.index)
        }
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        // Check that the ids yielded by this component are within the M31 range.
        assert!(
            LARGE_MEMORY_VALUE_ID_BASE + (self.index + 1) * ID_TO_BIG_MAX_ROWS - 1 < M31_P,
            "memory_id_to_big index {} would exceed M31 range",
            self.index
        );

        accumulate_constraints(
            component_data.trace_columns(),
            context,
            component_data,
            acc,
            self.index,
        );

        // Verify size <= ID_TO_BIG_MAX_ROWS (otherwise it will overlap with the next component)
        const { assert!(ID_TO_BIG_MAX_ROWS == (1 << MAX_SEQUENCE_LOG_SIZE)) };
        let max_sequence_log_size = usize::try_from(MAX_SEQUENCE_LOG_SIZE).unwrap();
        for bit_pos in (max_sequence_log_size + 1)..component_data.max_component_size_bits() {
            let bit = component_data.get_n_instances_bit(context, bit_pos);
            eq(context, bit, context.zero());
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
        None
    }
}
