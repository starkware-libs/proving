// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 1;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [multiplicity_0_col0] = input.try_into().unwrap();
    let pedersen_points_small_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_0".to_owned(),
    });
    let pedersen_points_small_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_1".to_owned(),
    });
    let pedersen_points_small_10 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_10".to_owned(),
    });
    let pedersen_points_small_11 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_11".to_owned(),
    });
    let pedersen_points_small_12 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_12".to_owned(),
    });
    let pedersen_points_small_13 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_13".to_owned(),
    });
    let pedersen_points_small_14 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_14".to_owned(),
    });
    let pedersen_points_small_15 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_15".to_owned(),
    });
    let pedersen_points_small_16 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_16".to_owned(),
    });
    let pedersen_points_small_17 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_17".to_owned(),
    });
    let pedersen_points_small_18 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_18".to_owned(),
    });
    let pedersen_points_small_19 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_19".to_owned(),
    });
    let pedersen_points_small_2 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_2".to_owned(),
    });
    let pedersen_points_small_20 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_20".to_owned(),
    });
    let pedersen_points_small_21 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_21".to_owned(),
    });
    let pedersen_points_small_22 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_22".to_owned(),
    });
    let pedersen_points_small_23 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_23".to_owned(),
    });
    let pedersen_points_small_24 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_24".to_owned(),
    });
    let pedersen_points_small_25 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_25".to_owned(),
    });
    let pedersen_points_small_26 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_26".to_owned(),
    });
    let pedersen_points_small_27 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_27".to_owned(),
    });
    let pedersen_points_small_28 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_28".to_owned(),
    });
    let pedersen_points_small_29 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_29".to_owned(),
    });
    let pedersen_points_small_3 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_3".to_owned(),
    });
    let pedersen_points_small_30 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_30".to_owned(),
    });
    let pedersen_points_small_31 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_31".to_owned(),
    });
    let pedersen_points_small_32 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_32".to_owned(),
    });
    let pedersen_points_small_33 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_33".to_owned(),
    });
    let pedersen_points_small_34 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_34".to_owned(),
    });
    let pedersen_points_small_35 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_35".to_owned(),
    });
    let pedersen_points_small_36 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_36".to_owned(),
    });
    let pedersen_points_small_37 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_37".to_owned(),
    });
    let pedersen_points_small_38 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_38".to_owned(),
    });
    let pedersen_points_small_39 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_39".to_owned(),
    });
    let pedersen_points_small_4 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_4".to_owned(),
    });
    let pedersen_points_small_40 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_40".to_owned(),
    });
    let pedersen_points_small_41 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_41".to_owned(),
    });
    let pedersen_points_small_42 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_42".to_owned(),
    });
    let pedersen_points_small_43 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_43".to_owned(),
    });
    let pedersen_points_small_44 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_44".to_owned(),
    });
    let pedersen_points_small_45 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_45".to_owned(),
    });
    let pedersen_points_small_46 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_46".to_owned(),
    });
    let pedersen_points_small_47 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_47".to_owned(),
    });
    let pedersen_points_small_48 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_48".to_owned(),
    });
    let pedersen_points_small_49 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_49".to_owned(),
    });
    let pedersen_points_small_5 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_5".to_owned(),
    });
    let pedersen_points_small_50 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_50".to_owned(),
    });
    let pedersen_points_small_51 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_51".to_owned(),
    });
    let pedersen_points_small_52 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_52".to_owned(),
    });
    let pedersen_points_small_53 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_53".to_owned(),
    });
    let pedersen_points_small_54 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_54".to_owned(),
    });
    let pedersen_points_small_55 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_55".to_owned(),
    });
    let pedersen_points_small_6 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_6".to_owned(),
    });
    let pedersen_points_small_7 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_7".to_owned(),
    });
    let pedersen_points_small_8 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_8".to_owned(),
    });
    let pedersen_points_small_9 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_9".to_owned(),
    });
    let seq_15 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_15".to_owned() });

    // Yield PedersenPointsTableWindowBits9.
    let tuple_0 = &[
        eval!(context, 1791500038),
        eval!(context, seq_15),
        eval!(context, pedersen_points_small_0),
        eval!(context, pedersen_points_small_1),
        eval!(context, pedersen_points_small_2),
        eval!(context, pedersen_points_small_3),
        eval!(context, pedersen_points_small_4),
        eval!(context, pedersen_points_small_5),
        eval!(context, pedersen_points_small_6),
        eval!(context, pedersen_points_small_7),
        eval!(context, pedersen_points_small_8),
        eval!(context, pedersen_points_small_9),
        eval!(context, pedersen_points_small_10),
        eval!(context, pedersen_points_small_11),
        eval!(context, pedersen_points_small_12),
        eval!(context, pedersen_points_small_13),
        eval!(context, pedersen_points_small_14),
        eval!(context, pedersen_points_small_15),
        eval!(context, pedersen_points_small_16),
        eval!(context, pedersen_points_small_17),
        eval!(context, pedersen_points_small_18),
        eval!(context, pedersen_points_small_19),
        eval!(context, pedersen_points_small_20),
        eval!(context, pedersen_points_small_21),
        eval!(context, pedersen_points_small_22),
        eval!(context, pedersen_points_small_23),
        eval!(context, pedersen_points_small_24),
        eval!(context, pedersen_points_small_25),
        eval!(context, pedersen_points_small_26),
        eval!(context, pedersen_points_small_27),
        eval!(context, pedersen_points_small_28),
        eval!(context, pedersen_points_small_29),
        eval!(context, pedersen_points_small_30),
        eval!(context, pedersen_points_small_31),
        eval!(context, pedersen_points_small_32),
        eval!(context, pedersen_points_small_33),
        eval!(context, pedersen_points_small_34),
        eval!(context, pedersen_points_small_35),
        eval!(context, pedersen_points_small_36),
        eval!(context, pedersen_points_small_37),
        eval!(context, pedersen_points_small_38),
        eval!(context, pedersen_points_small_39),
        eval!(context, pedersen_points_small_40),
        eval!(context, pedersen_points_small_41),
        eval!(context, pedersen_points_small_42),
        eval!(context, pedersen_points_small_43),
        eval!(context, pedersen_points_small_44),
        eval!(context, pedersen_points_small_45),
        eval!(context, pedersen_points_small_46),
        eval!(context, pedersen_points_small_47),
        eval!(context, pedersen_points_small_48),
        eval!(context, pedersen_points_small_49),
        eval!(context, pedersen_points_small_50),
        eval!(context, pedersen_points_small_51),
        eval!(context, pedersen_points_small_52),
        eval!(context, pedersen_points_small_53),
        eval!(context, pedersen_points_small_54),
        eval!(context, pedersen_points_small_55),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "pedersen_points_table_window_bits_9".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 15 rows
        let size_bit = component_data.get_n_instances_bit(context, 15);
        eq(context, size_bit, context.one());
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
        Some(15)
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use circuits::context::Context;
    use circuits::ivalue::qm31_from_u32s;
    use circuits_stark_verifier::constraint_eval::*;
    use circuits_stark_verifier::test_utils::TestComponentData;
    use stwo::core::fields::qm31::QM31;

    use super::Component;
    #[allow(unused_imports)]
    use crate::components::prelude::PreProcessedColumnId;
    use crate::sample_evaluations::*;

    #[test]
    fn test_evaluation_result() {
        let component = Component {};
        let mut context: Context<QM31> = Default::default();
        context.enable_assert_eq_on_eval();
        let trace_columns = [qm31_from_u32s(1659099300, 905558730, 651199673, 1375009625)];
        let interaction_columns = [qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119)];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            32768,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "seq_15".to_owned() },
                context.constant(qm31_from_u32s(1561133224, 586108960, 1063114695, 1758380471)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_0".to_owned() },
                context.constant(qm31_from_u32s(1071331976, 281835572, 1927130496, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_1".to_owned() },
                context.constant(qm31_from_u32s(1138441155, 416053300, 1994239360, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_2".to_owned() },
                context.constant(qm31_from_u32s(937113618, 13400116, 1792912768, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_3".to_owned() },
                context.constant(qm31_from_u32s(1004222797, 147617844, 1860021632, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_4".to_owned() },
                context.constant(qm31_from_u32s(1339768692, 818706484, 48082305, 1936455920)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_5".to_owned() },
                context.constant(qm31_from_u32s(1406877871, 952924212, 115191169, 1936455920)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_6".to_owned() },
                context.constant(qm31_from_u32s(1205550334, 550271028, 2061348224, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_7".to_owned() },
                context.constant(qm31_from_u32s(1272659513, 684488756, 2128457088, 1936455919)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_8".to_owned() },
                context.constant(qm31_from_u32s(1608205408, 1355577396, 316517761, 1936455920)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_9".to_owned() },
                context.constant(qm31_from_u32s(1675314587, 1489795124, 383626625, 1936455920)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_10".to_owned() },
                context.constant(qm31_from_u32s(943093592, 1811147488, 1730181953, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_11".to_owned() },
                context.constant(qm31_from_u32s(875984413, 1676929760, 1663073089, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_12".to_owned() },
                context.constant(qm31_from_u32s(1077311950, 2079582944, 1864399681, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_13".to_owned() },
                context.constant(qm31_from_u32s(1010202771, 1945365216, 1797290817, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_14".to_owned() },
                context.constant(qm31_from_u32s(1211530308, 200534753, 1998617410, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_15".to_owned() },
                context.constant(qm31_from_u32s(1144421129, 66317025, 1931508546, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_16".to_owned() },
                context.constant(qm31_from_u32s(1345748666, 468970209, 2132835138, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_17".to_owned() },
                context.constant(qm31_from_u32s(1278639487, 334752481, 2065726274, 2068794068)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_18".to_owned() },
                context.constant(qm31_from_u32s(1479967024, 737405665, 119569219, 2068794069)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_19".to_owned() },
                context.constant(qm31_from_u32s(1412857845, 603187937, 52460355, 2068794069)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_20".to_owned() },
                context.constant(qm31_from_u32s(808568049, 1542711852, 1595964045, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_21".to_owned() },
                context.constant(qm31_from_u32s(875677228, 1676929580, 1663072909, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_22".to_owned() },
                context.constant(qm31_from_u32s(942786407, 1811147308, 1730181773, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_23".to_owned() },
                context.constant(qm31_from_u32s(1009895586, 1945365036, 1797290637, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_24".to_owned() },
                context.constant(qm31_from_u32s(1077004765, 2079582764, 1864399501, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_25".to_owned() },
                context.constant(qm31_from_u32s(1144113944, 66316845, 1931508366, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_26".to_owned() },
                context.constant(qm31_from_u32s(1211223123, 200534573, 1998617230, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_27".to_owned() },
                context.constant(qm31_from_u32s(1278332302, 334752301, 2065726094, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_28".to_owned() },
                context.constant(qm31_from_u32s(1345441481, 468970029, 2132834958, 2068794008)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_29".to_owned() },
                context.constant(qm31_from_u32s(1412550660, 603187757, 52460175, 2068794009)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_30".to_owned() },
                context.constant(qm31_from_u32s(137588735, 200534635, 924875468, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_31".to_owned() },
                context.constant(qm31_from_u32s(70479556, 66316907, 857766604, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_32".to_owned() },
                context.constant(qm31_from_u32s(3370377, 2079582826, 790657739, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_33".to_owned() },
                context.constant(qm31_from_u32s(2083744845, 1945365097, 723548875, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_34".to_owned() },
                context.constant(qm31_from_u32s(406025451, 737405547, 1193310924, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_35".to_owned() },
                context.constant(qm31_from_u32s(338916272, 603187819, 1126202060, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_36".to_owned() },
                context.constant(qm31_from_u32s(271807093, 468970091, 1059093196, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_37".to_owned() },
                context.constant(qm31_from_u32s(204697914, 334752363, 991984332, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_38".to_owned() },
                context.constant(qm31_from_u32s(1748198950, 1274276457, 388004555, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_39".to_owned() },
                context.constant(qm31_from_u32s(1681089771, 1140058729, 320895691, 2068794029)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_40".to_owned() },
                context.constant(qm31_from_u32s(1077619135, 2079583124, 1864399861, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_41".to_owned() },
                context.constant(qm31_from_u32s(1144728314, 66317205, 1931508726, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_42".to_owned() },
                context.constant(qm31_from_u32s(943400777, 1811147668, 1730182133, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_43".to_owned() },
                context.constant(qm31_from_u32s(1010509956, 1945365396, 1797290997, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_44".to_owned() },
                context.constant(qm31_from_u32s(1346055851, 468970389, 2132835318, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_45".to_owned() },
                context.constant(qm31_from_u32s(1413165030, 603188117, 52460535, 2068794129)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_46".to_owned() },
                context.constant(qm31_from_u32s(1211837493, 200534933, 1998617590, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_47".to_owned() },
                context.constant(qm31_from_u32s(1278946672, 334752661, 2065726454, 2068794128)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_48".to_owned() },
                context.constant(qm31_from_u32s(1614492567, 1005841301, 253787127, 2068794129)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_49".to_owned() },
                context.constant(qm31_from_u32s(1681601746, 1140059029, 320895991, 2068794129)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_50".to_owned() },
                context.constant(qm31_from_u32s(406639821, 737405907, 1193311284, 2068794149)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_51".to_owned() },
                context.constant(qm31_from_u32s(339530642, 603188179, 1126202420, 2068794149)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_52".to_owned() },
                context.constant(qm31_from_u32s(540858179, 1005841363, 1327529012, 2068794149)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_53".to_owned() },
                context.constant(qm31_from_u32s(473749000, 871623635, 1260420148, 2068794149)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_54".to_owned() },
                context.constant(qm31_from_u32s(138203105, 200534995, 924875828, 2068794149)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_small_55".to_owned() },
                context.constant(qm31_from_u32s(71093926, 66317267, 857766964, 2068794149)),
            ),
        ]);
        let public_params = HashMap::from([]);
        let mut accumulator = CompositionConstraintAccumulator::new(
            &mut context,
            preprocessed_columns,
            public_params,
            random_coeff,
            interaction_elements,
        );
        component.evaluate(&mut context, &component_data, &mut accumulator);
        let claimed_sum =
            context.new_var(qm31_from_u32s(1398335417, 314974026, 1722107152, 821933968));
        accumulator.finalize_logup_in_pairs(
            &mut context,
            <TestComponentData as ComponentDataTrait<QM31>>::interaction_columns(&component_data),
            &component_data,
            claimed_sum,
        );

        let result = accumulator.finalize();
        let result_value = context.get(result);
        assert_eq!(result_value, PEDERSEN_POINTS_TABLE_WINDOW_BITS_9_SAMPLE_EVAL_RESULT)
    }
}
