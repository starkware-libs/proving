use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::partial_round::*;

#[test]
fn test_poseidon_partial_round() {
    let air_fn = Poseidon3PartialRoundsChain {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(4),
            [
                const_felt252_expr!(
                    0xe3537dca9f28533cade50a1864816c70u128,
                    0x9b4e4d358f1423380738337bbf67c8u128
                )
                .into(),
                const_felt252_expr!(
                    0xe243fdbf6366aed9d5369d50e2b9b480u128,
                    0x3c4e1f55af4447cadf6bd704eb01d79u128
                )
                .into(),
                const_felt252_expr!(
                    0xa07a6ff0d70f8d1338144fd09791059du128,
                    0x243f648e2ecd9720bb2657aad0a79b5u128
                )
                .into(),
                const_felt252_expr!(
                    0xae15579d749d434bece9fb13fa3a39d7u128,
                    0x396488f7b22e929802a48afea919080u128
                )
                .into(),
            ],
        ),
    );
    let expected_output = [
        const_felt252_expr!(
            0x8c0a712f0be18a3795c645569683af77u128,
            0x6533e92d5aa8e4f6867494ec59573cbu128
        ),
        const_felt252_expr!(
            0xc48f1f967e974e894689a7d15f60af10u128,
            0x37292a0a739ee1dc8b6b27cc4e2a6a4u128
        ),
        const_felt252_expr!(
            0x13a7f72654da48fb022be337f1179920u128,
            0x6e43f5086c021c40056b2673a6a9a4cu128
        ),
        const_felt252_expr!(
            0xef171642498d0042106c34abdf2c6b5du128,
            0x63c745ffb88adea76b5f49ce227847fu128
        ),
    ];
    for (out, exp_out) in output.2.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }
    expect![[r#"
        (1, "enabler"),
        (0, "input_limb_0"),
        (4, "input_limb_1"),
        (75590768, "input_limb_2"),
        (77677324, "input_limb_3"),
        (21820087, "input_limb_4"),
        (115691412, "input_limb_5"),
        (76428599, "input_limb_6"),
        (116883151, "input_limb_7"),
        (100781580, "input_limb_8"),
        (75014425, "input_limb_9"),
        (55463221, "input_limb_10"),
        (19, "input_limb_11"),
        (45724800, "input_limb_12"),
        (114534940, "input_limb_13"),
        (45836116, "input_limb_14"),
        (115323315, "input_limb_15"),
        (127804479, "input_limb_16"),
        (10313786, "input_limb_17"),
        (58568540, "input_limb_18"),
        (128066533, "input_limb_19"),
        (81917274, "input_limb_20"),
        (120, "input_limb_21"),
        (126944669, "input_limb_22"),
        (42596882, "input_limb_23"),
        (104090848, "input_limb_24"),
        (133720967, "input_limb_25"),
        (56231846, "input_limb_26"),
        (89789683, "input_limb_27"),
        (49060190, "input_limb_28"),
        (124177296, "input_limb_29"),
        (66472162, "input_limb_30"),
        (72, "input_limb_31"),
        (37370327, "input_limb_32"),
        (88040063, "input_limb_33"),
        (84750259, "input_limb_34"),
        (63879758, "input_limb_35"),
        (713045, "input_limb_36"),
        (131408673, "input_limb_37"),
        (692779, "input_limb_38"),
        (18303308, "input_limb_39"),
        (105418619, "input_limb_40"),
        (114, "input_limb_41"),
        (39976036, "poseidon_round_keys_output_limb_0"),
        (125368084, "poseidon_round_keys_output_limb_1"),
        (90162789, "poseidon_round_keys_output_limb_2"),
        (21698025, "poseidon_round_keys_output_limb_3"),
        (32223256, "poseidon_round_keys_output_limb_4"),
        (55515373, "poseidon_round_keys_output_limb_5"),
        (102746996, "poseidon_round_keys_output_limb_6"),
        (33190869, "poseidon_round_keys_output_limb_7"),
        (47808605, "poseidon_round_keys_output_limb_8"),
        (241, "poseidon_round_keys_output_limb_9"),
        (8056734, "poseidon_round_keys_output_limb_10"),
        (38977152, "poseidon_round_keys_output_limb_11"),
        (21774454, "poseidon_round_keys_output_limb_12"),
        (70696684, "poseidon_round_keys_output_limb_13"),
        (111793298, "poseidon_round_keys_output_limb_14"),
        (92929314, "poseidon_round_keys_output_limb_15"),
        (112520153, "poseidon_round_keys_output_limb_16"),
        (104842155, "poseidon_round_keys_output_limb_17"),
        (21493055, "poseidon_round_keys_output_limb_18"),
        (203, "poseidon_round_keys_output_limb_19"),
        (44075553, "poseidon_round_keys_output_limb_20"),
        (111547962, "poseidon_round_keys_output_limb_21"),
        (77453055, "poseidon_round_keys_output_limb_22"),
        (104012182, "poseidon_round_keys_output_limb_23"),
        (40046921, "poseidon_round_keys_output_limb_24"),
        (134143042, "poseidon_round_keys_output_limb_25"),
        (111559922, "poseidon_round_keys_output_limb_26"),
        (10704274, "poseidon_round_keys_output_limb_27"),
        (102956839, "poseidon_round_keys_output_limb_28"),
        (0, "poseidon_round_keys_output_limb_29"),
        (91212196, "cube_252_output_limb_0"),
        (23358383, "cube_252_output_limb_1"),
        (19214592, "cube_252_output_limb_2"),
        (1563360, "cube_252_output_limb_3"),
        (46375875, "cube_252_output_limb_4"),
        (119343394, "cube_252_output_limb_5"),
        (80654758, "cube_252_output_limb_6"),
        (123261739, "cube_252_output_limb_7"),
        (114399052, "cube_252_output_limb_8"),
        (53, "cube_252_output_limb_9"),
        (89692203, "combination_limb_0"),
        (52313223, "combination_limb_1"),
        (110052674, "combination_limb_2"),
        (104847782, "combination_limb_3"),
        (45490686, "combination_limb_4"),
        (19803514, "combination_limb_5"),
        (19140353, "combination_limb_6"),
        (51648321, "combination_limb_7"),
        (87061184, "combination_limb_8"),
        (70, "combination_limb_9"),
        (3, "p_coef"),
        (45166678, "combination_limb_0"),
        (104626447, "combination_limb_1"),
        (85887620, "combination_limb_2"),
        (75477837, "combination_limb_3"),
        (90981373, "combination_limb_4"),
        (39607028, "combination_limb_5"),
        (38280706, "combination_limb_6"),
        (103296642, "combination_limb_7"),
        (39904640, "combination_limb_8"),
        (141, "combination_limb_9"),
        (0, "p_coef"),
        (109293431, "cube_252_output_limb_0"),
        (13150930, "cube_252_output_limb_1"),
        (103341655, "cube_252_output_limb_2"),
        (9930224, "cube_252_output_limb_3"),
        (79216807, "cube_252_output_limb_4"),
        (93006567, "cube_252_output_limb_5"),
        (35246675, "cube_252_output_limb_6"),
        (89420411, "cube_252_output_limb_7"),
        (54432469, "cube_252_output_limb_8"),
        (202, "cube_252_output_limb_9"),
        (128997256, "combination_limb_0"),
        (10124565, "combination_limb_1"),
        (110957197, "combination_limb_2"),
        (132489125, "combination_limb_3"),
        (86385784, "combination_limb_4"),
        (80011942, "combination_limb_5"),
        (18273871, "combination_limb_6"),
        (82294903, "combination_limb_7"),
        (21581907, "combination_limb_8"),
        (55, "combination_limb_9"),
        (3, "p_coef"),
        (123776784, "combination_limb_0"),
        (20249131, "combination_limb_1"),
        (87696666, "combination_limb_2"),
        (130760523, "combination_limb_3"),
        (38553841, "combination_limb_4"),
        (25806157, "combination_limb_5"),
        (36547743, "combination_limb_6"),
        (30372078, "combination_limb_7"),
        (43163815, "combination_limb_8"),
        (110, "combination_limb_9"),
        (0, "p_coef"),
        (18323744, "cube_252_output_limb_0"),
        (92038910, "cube_252_output_limb_1"),
        (19131400, "cube_252_output_limb_2"),
        (59976301, "cube_252_output_limb_3"),
        (79772287, "cube_252_output_limb_4"),
        (108320052, "cube_252_output_limb_5"),
        (1420441, "cube_252_output_limb_6"),
        (100732448, "cube_252_output_limb_7"),
        (71258246, "cube_252_output_limb_8"),
        (220, "cube_252_output_limb_9"),
        (127284655, "combination_limb_0"),
        (113461949, "combination_limb_1"),
        (33588256, "combination_limb_2"),
        (93360739, "combination_limb_3"),
        (67074232, "combination_limb_4"),
        (81930116, "combination_limb_5"),
        (114736787, "combination_limb_6"),
        (102938605, "combination_limb_7"),
        (104476669, "combination_limb_8"),
        (227, "combination_limb_9"),
        (3, "p_coef"),
        (120351581, "combination_limb_0"),
        (92706171, "combination_limb_1"),
        (67176513, "combination_limb_2"),
        (52503750, "combination_limb_3"),
        (134148465, "combination_limb_4"),
        (29642504, "combination_limb_5"),
        (95255847, "combination_limb_6"),
        (71659347, "combination_limb_7"),
        (74735611, "combination_limb_8"),
        (199, "combination_limb_9"),
        (1, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(30),
            [
                const_felt252_expr!(
                    0x1d810ad08ec7292a3e480d064e3c33fdu128,
                    0x24197cdb19d911de8c1aefd79a1e361u128
                )
                .into(),
                const_felt252_expr!(
                    0x3df9d9c8f0ea31661422c910a953e90eu128,
                    0x79bc5f0ef7295007407214b31fc9c5du128
                )
                .into(),
                const_felt252_expr!(
                    0x37f0afa4744515e8b6289c6e7088fa53u128,
                    0x5fede0e8c1d858b07208faaf87e2825u128
                )
                .into(),
                const_felt252_expr!(
                    0xd990717ed1feae68b4ce9a884f5fa1dcu128,
                    0x124ab89fe982b447d1721cab0feeeddu128
                )
                .into(),
            ],
        ),
    );
    let expected_output = [
        const_felt252_expr!(
            0xb6f492c4e386fea4cae83dcb7d09dcfau128,
            0x11737e2519e23a2efbeb74703e14806u128
        ),
        const_felt252_expr!(
            0x19c06386a2f3c99c45a26f4bb3484bb7u128,
            0x681bfdd9318d1399d41f9a200353addu128
        ),
        const_felt252_expr!(
            0xc9509dbbe4d81cc3bda85be223cbab69u128,
            0x6446e7317d7f5c81b361fa0a425b38fu128
        ),
        const_felt252_expr!(
            0x9e0c2c87b68a3eb97700c355bfbfdf01u128,
            0x3c04b83fb3af01feceeb140bbab0322u128
        ),
    ];
    for (out, exp_out) in output.2.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }
}
