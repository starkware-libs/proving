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
    let pedersen_points_0 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_0".to_owned() });
    let pedersen_points_1 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_1".to_owned() });
    let pedersen_points_10 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_10".to_owned() });
    let pedersen_points_11 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_11".to_owned() });
    let pedersen_points_12 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_12".to_owned() });
    let pedersen_points_13 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_13".to_owned() });
    let pedersen_points_14 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_14".to_owned() });
    let pedersen_points_15 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_15".to_owned() });
    let pedersen_points_16 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_16".to_owned() });
    let pedersen_points_17 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_17".to_owned() });
    let pedersen_points_18 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_18".to_owned() });
    let pedersen_points_19 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_19".to_owned() });
    let pedersen_points_2 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_2".to_owned() });
    let pedersen_points_20 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_20".to_owned() });
    let pedersen_points_21 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_21".to_owned() });
    let pedersen_points_22 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_22".to_owned() });
    let pedersen_points_23 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_23".to_owned() });
    let pedersen_points_24 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_24".to_owned() });
    let pedersen_points_25 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_25".to_owned() });
    let pedersen_points_26 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_26".to_owned() });
    let pedersen_points_27 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_27".to_owned() });
    let pedersen_points_28 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_28".to_owned() });
    let pedersen_points_29 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_29".to_owned() });
    let pedersen_points_3 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_3".to_owned() });
    let pedersen_points_30 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_30".to_owned() });
    let pedersen_points_31 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_31".to_owned() });
    let pedersen_points_32 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_32".to_owned() });
    let pedersen_points_33 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_33".to_owned() });
    let pedersen_points_34 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_34".to_owned() });
    let pedersen_points_35 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_35".to_owned() });
    let pedersen_points_36 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_36".to_owned() });
    let pedersen_points_37 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_37".to_owned() });
    let pedersen_points_38 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_38".to_owned() });
    let pedersen_points_39 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_39".to_owned() });
    let pedersen_points_4 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_4".to_owned() });
    let pedersen_points_40 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_40".to_owned() });
    let pedersen_points_41 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_41".to_owned() });
    let pedersen_points_42 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_42".to_owned() });
    let pedersen_points_43 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_43".to_owned() });
    let pedersen_points_44 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_44".to_owned() });
    let pedersen_points_45 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_45".to_owned() });
    let pedersen_points_46 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_46".to_owned() });
    let pedersen_points_47 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_47".to_owned() });
    let pedersen_points_48 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_48".to_owned() });
    let pedersen_points_49 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_49".to_owned() });
    let pedersen_points_5 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_5".to_owned() });
    let pedersen_points_50 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_50".to_owned() });
    let pedersen_points_51 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_51".to_owned() });
    let pedersen_points_52 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_52".to_owned() });
    let pedersen_points_53 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_53".to_owned() });
    let pedersen_points_54 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_54".to_owned() });
    let pedersen_points_55 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_55".to_owned() });
    let pedersen_points_6 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_6".to_owned() });
    let pedersen_points_7 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_7".to_owned() });
    let pedersen_points_8 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_8".to_owned() });
    let pedersen_points_9 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_9".to_owned() });
    let seq_23 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_23".to_owned() });

    // Yield PedersenPointsTableWindowBits18.
    let tuple_0 = &[
        eval!(context, 1444721856),
        eval!(context, seq_23),
        eval!(context, pedersen_points_0),
        eval!(context, pedersen_points_1),
        eval!(context, pedersen_points_2),
        eval!(context, pedersen_points_3),
        eval!(context, pedersen_points_4),
        eval!(context, pedersen_points_5),
        eval!(context, pedersen_points_6),
        eval!(context, pedersen_points_7),
        eval!(context, pedersen_points_8),
        eval!(context, pedersen_points_9),
        eval!(context, pedersen_points_10),
        eval!(context, pedersen_points_11),
        eval!(context, pedersen_points_12),
        eval!(context, pedersen_points_13),
        eval!(context, pedersen_points_14),
        eval!(context, pedersen_points_15),
        eval!(context, pedersen_points_16),
        eval!(context, pedersen_points_17),
        eval!(context, pedersen_points_18),
        eval!(context, pedersen_points_19),
        eval!(context, pedersen_points_20),
        eval!(context, pedersen_points_21),
        eval!(context, pedersen_points_22),
        eval!(context, pedersen_points_23),
        eval!(context, pedersen_points_24),
        eval!(context, pedersen_points_25),
        eval!(context, pedersen_points_26),
        eval!(context, pedersen_points_27),
        eval!(context, pedersen_points_28),
        eval!(context, pedersen_points_29),
        eval!(context, pedersen_points_30),
        eval!(context, pedersen_points_31),
        eval!(context, pedersen_points_32),
        eval!(context, pedersen_points_33),
        eval!(context, pedersen_points_34),
        eval!(context, pedersen_points_35),
        eval!(context, pedersen_points_36),
        eval!(context, pedersen_points_37),
        eval!(context, pedersen_points_38),
        eval!(context, pedersen_points_39),
        eval!(context, pedersen_points_40),
        eval!(context, pedersen_points_41),
        eval!(context, pedersen_points_42),
        eval!(context, pedersen_points_43),
        eval!(context, pedersen_points_44),
        eval!(context, pedersen_points_45),
        eval!(context, pedersen_points_46),
        eval!(context, pedersen_points_47),
        eval!(context, pedersen_points_48),
        eval!(context, pedersen_points_49),
        eval!(context, pedersen_points_50),
        eval!(context, pedersen_points_51),
        eval!(context, pedersen_points_52),
        eval!(context, pedersen_points_53),
        eval!(context, pedersen_points_54),
        eval!(context, pedersen_points_55),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "pedersen_points_table_window_bits_18".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 23 rows
        let size_bit = component_data.get_n_instances_bit(context, 23);
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
        Some(23)
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
            8388608,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "seq_23".to_owned() },
                context.constant(qm31_from_u32s(1427222051, 317673684, 928897147, 1758380531)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_0".to_owned() },
                context.constant(qm31_from_u32s(1498683260, 701261032, 9432172, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_1".to_owned() },
                context.constant(qm31_from_u32s(1565792439, 835478760, 76541036, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_2".to_owned() },
                context.constant(qm31_from_u32s(1364464902, 432825576, 2022698091, 1479828961)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_3".to_owned() },
                context.constant(qm31_from_u32s(1431574081, 567043304, 2089806955, 1479828961)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_4".to_owned() },
                context.constant(qm31_from_u32s(1767119976, 1238131944, 277867628, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_5".to_owned() },
                context.constant(qm31_from_u32s(1834229155, 1372349672, 344976492, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_6".to_owned() },
                context.constant(qm31_from_u32s(1632901618, 969696488, 143649900, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_7".to_owned() },
                context.constant(qm31_from_u32s(1700010797, 1103914216, 210758764, 1479828962)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_8".to_owned() },
                context.constant(qm31_from_u32s(961809828, 1775002855, 1620044906, 1479828961)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_9".to_owned() },
                context.constant(qm31_from_u32s(1028919007, 1909220583, 1687153770, 1479828961)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_10".to_owned() },
                context.constant(qm31_from_u32s(1362728643, 314406152, 1163211253, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_11".to_owned() },
                context.constant(qm31_from_u32s(1295619464, 180188424, 1096102389, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_12".to_owned() },
                context.constant(qm31_from_u32s(1496947001, 582841608, 1297428981, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_13".to_owned() },
                context.constant(qm31_from_u32s(1429837822, 448623880, 1230320117, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_14".to_owned() },
                context.constant(qm31_from_u32s(1631165359, 851277064, 1431646709, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_15".to_owned() },
                context.constant(qm31_from_u32s(1564056180, 717059336, 1364537845, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_16".to_owned() },
                context.constant(qm31_from_u32s(1765383717, 1119712520, 1565864437, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_17".to_owned() },
                context.constant(qm31_from_u32s(1698274538, 985494792, 1498755573, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_18".to_owned() },
                context.constant(qm31_from_u32s(825855211, 1388147975, 626340340, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_19".to_owned() },
                context.constant(qm31_from_u32s(758746032, 1253930247, 559231476, 1597535363)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_20".to_owned() },
                context.constant(qm31_from_u32s(1228203100, 45970516, 1028993345, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_21".to_owned() },
                context.constant(qm31_from_u32s(1295312279, 180188244, 1096102209, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_22".to_owned() },
                context.constant(qm31_from_u32s(1362421458, 314405972, 1163211073, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_23".to_owned() },
                context.constant(qm31_from_u32s(1429530637, 448623700, 1230319937, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_24".to_owned() },
                context.constant(qm31_from_u32s(1496639816, 582841428, 1297428801, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_25".to_owned() },
                context.constant(qm31_from_u32s(1563748995, 717059156, 1364537665, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_26".to_owned() },
                context.constant(qm31_from_u32s(1630858174, 851276884, 1431646529, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_27".to_owned() },
                context.constant(qm31_from_u32s(1697967353, 985494612, 1498755393, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_28".to_owned() },
                context.constant(qm31_from_u32s(691329668, 1119712339, 492122432, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_29".to_owned() },
                context.constant(qm31_from_u32s(758438847, 1253930067, 559231296, 1597535303)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_30".to_owned() },
                context.constant(qm31_from_u32s(557213705, 851276943, 357904764, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_31".to_owned() },
                context.constant(qm31_from_u32s(490104526, 717059215, 290795900, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_32".to_owned() },
                context.constant(qm31_from_u32s(422995347, 582841487, 223687036, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_33".to_owned() },
                context.constant(qm31_from_u32s(355886168, 448623759, 156578172, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_34".to_owned() },
                context.constant(qm31_from_u32s(825650421, 1388147855, 626340220, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_35".to_owned() },
                context.constant(qm31_from_u32s(758541242, 1253930127, 559231356, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_36".to_owned() },
                context.constant(qm31_from_u32s(691432063, 1119712399, 492122492, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_37".to_owned() },
                context.constant(qm31_from_u32s(624322884, 985494671, 425013628, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_38".to_owned() },
                context.constant(qm31_from_u32s(1094087137, 1925018767, 894775676, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_39".to_owned() },
                context.constant(qm31_from_u32s(1026977958, 1790801039, 827666812, 1597535323)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_40".to_owned() },
                context.constant(qm31_from_u32s(1497254186, 582841788, 1297429161, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_41".to_owned() },
                context.constant(qm31_from_u32s(1564363365, 717059516, 1364538025, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_42".to_owned() },
                context.constant(qm31_from_u32s(1363035828, 314406332, 1163211433, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_43".to_owned() },
                context.constant(qm31_from_u32s(1430145007, 448624060, 1230320297, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_44".to_owned() },
                context.constant(qm31_from_u32s(1765690902, 1119712700, 1565864617, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_45".to_owned() },
                context.constant(qm31_from_u32s(1832800081, 1253930428, 1632973481, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_46".to_owned() },
                context.constant(qm31_from_u32s(1631472544, 851277244, 1431646889, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_47".to_owned() },
                context.constant(qm31_from_u32s(1698581723, 985494972, 1498755753, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_48".to_owned() },
                context.constant(qm31_from_u32s(960380754, 1656583611, 760558248, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_49".to_owned() },
                context.constant(qm31_from_u32s(1027489933, 1790801339, 827667112, 1597535423)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_50".to_owned() },
                context.constant(qm31_from_u32s(826264791, 1388148215, 626340580, 1597535443)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_51".to_owned() },
                context.constant(qm31_from_u32s(759155612, 1253930487, 559231716, 1597535443)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_52".to_owned() },
                context.constant(qm31_from_u32s(960483149, 1656583671, 760558308, 1597535443)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_53".to_owned() },
                context.constant(qm31_from_u32s(893373970, 1522365943, 693449444, 1597535443)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_54".to_owned() },
                context.constant(qm31_from_u32s(557828075, 851277303, 357905124, 1597535443)),
            ),
            (
                PreProcessedColumnId { id: "pedersen_points_55".to_owned() },
                context.constant(qm31_from_u32s(490718896, 717059575, 290796260, 1597535443)),
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
        assert_eq!(result_value, PEDERSEN_POINTS_TABLE_WINDOW_BITS_18_SAMPLE_EVAL_RESULT)
    }
}
