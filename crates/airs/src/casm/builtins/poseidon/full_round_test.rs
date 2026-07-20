use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::full_round::*;

#[test]
fn test_poseidon_full_round() {
    let air_fn = PoseidonFullRoundChain {};
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let (state, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(0),
            [
                const_felt252_expr!(
                    0xe58e2ad98109ae4780b7fd8eac77fe70u128,
                    0x6861759ea556a2339dd92f9562a30b9u128
                )
                .into(),
                const_felt252_expr!(
                    0x3da43f76abf28a64e4ab1a22f27508c6u128,
                    0x3827681995d5af9ffc8397a3d00425au128
                )
                .into(),
                const_felt252_expr!(
                    0x2cac75dc279b2d687a0dbe17704a830cu128,
                    0x3a3956d2fad44d0e7f760a2277dc7cbu128
                )
                .into(),
            ],
        ),
    );
    let expected_output = [
        const_felt252_expr!(
            0x182ac04678b725e9cd28a9a910551228u128,
            0x7fe46959f384f2a87db105d7a8ef27u128
        ),
        const_felt252_expr!(
            0xb9da7ad31613d5fc91092ddda81c074eu128,
            0xbda37a4df11995b73b227acf31008cu128
        ),
        const_felt252_expr!(
            0x269c04ea49158efba215b84ebe38196cu128,
            0x2a756b59d27dc9eacc79f2ec1a3cbceu128
        ),
    ];
    for (out, exp_out) in output.2.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }
    expect![[r#"
        (1, "enabler"),
        (0, "input_limb_0"),
        (0, "input_limb_1"),
        (74972784, "input_limb_2"),
        (117420501, "input_limb_3"),
        (112795138, "input_limb_4"),
        (91013252, "input_limb_5"),
        (60709090, "input_limb_6"),
        (44848225, "input_limb_7"),
        (108487870, "input_limb_8"),
        (44781849, "input_limb_9"),
        (102193642, "input_limb_10"),
        (208, "input_limb_11"),
        (41224390, "input_limb_12"),
        (90391646, "input_limb_13"),
        (36279186, "input_limb_14"),
        (129717753, "input_limb_15"),
        (94624323, "input_limb_16"),
        (75104388, "input_limb_17"),
        (133303902, "input_limb_18"),
        (48945103, "input_limb_19"),
        (41320857, "input_limb_20"),
        (112, "input_limb_21"),
        (4883212, "input_limb_22"),
        (28820206, "input_limb_23"),
        (79012328, "input_limb_24"),
        (49157069, "input_limb_25"),
        (78826183, "input_limb_26"),
        (72285071, "input_limb_27"),
        (33413160, "input_limb_28"),
        (90842759, "input_limb_29"),
        (60124463, "input_limb_30"),
        (116, "input_limb_31"),
        (51082887, "cube_252_output_limb_0"),
        (132526683, "cube_252_output_limb_1"),
        (40577028, "cube_252_output_limb_2"),
        (114704386, "cube_252_output_limb_3"),
        (5679346, "cube_252_output_limb_4"),
        (6068304, "cube_252_output_limb_5"),
        (118695257, "cube_252_output_limb_6"),
        (129708921, "cube_252_output_limb_7"),
        (132446121, "cube_252_output_limb_8"),
        (226, "cube_252_output_limb_9"),
        (47480788, "cube_252_output_limb_0"),
        (133163785, "cube_252_output_limb_1"),
        (25132734, "cube_252_output_limb_2"),
        (71005217, "cube_252_output_limb_3"),
        (44853823, "cube_252_output_limb_4"),
        (94594251, "cube_252_output_limb_5"),
        (73957394, "cube_252_output_limb_6"),
        (14290763, "cube_252_output_limb_7"),
        (43794190, "cube_252_output_limb_8"),
        (175, "cube_252_output_limb_9"),
        (98515447, "cube_252_output_limb_0"),
        (23887113, "cube_252_output_limb_1"),
        (9410118, "cube_252_output_limb_2"),
        (48788706, "cube_252_output_limb_3"),
        (19827660, "cube_252_output_limb_4"),
        (11767508, "cube_252_output_limb_5"),
        (70151043, "cube_252_output_limb_6"),
        (92242292, "cube_252_output_limb_7"),
        (114608336, "cube_252_output_limb_8"),
        (242, "cube_252_output_limb_9"),
        (108983501, "poseidon_round_keys_output_limb_0"),
        (67515900, "poseidon_round_keys_output_limb_1"),
        (54991392, "poseidon_round_keys_output_limb_2"),
        (75273041, "poseidon_round_keys_output_limb_3"),
        (93491655, "poseidon_round_keys_output_limb_4"),
        (71472462, "poseidon_round_keys_output_limb_5"),
        (72290464, "poseidon_round_keys_output_limb_6"),
        (34668303, "poseidon_round_keys_output_limb_7"),
        (113539709, "poseidon_round_keys_output_limb_8"),
        (196, "poseidon_round_keys_output_limb_9"),
        (33937062, "poseidon_round_keys_output_limb_10"),
        (130217817, "poseidon_round_keys_output_limb_11"),
        (98349751, "poseidon_round_keys_output_limb_12"),
        (132532806, "poseidon_round_keys_output_limb_13"),
        (32690983, "poseidon_round_keys_output_limb_14"),
        (36806568, "poseidon_round_keys_output_limb_15"),
        (116766677, "poseidon_round_keys_output_limb_16"),
        (52963354, "poseidon_round_keys_output_limb_17"),
        (25557217, "poseidon_round_keys_output_limb_18"),
        (241, "poseidon_round_keys_output_limb_19"),
        (68589311, "poseidon_round_keys_output_limb_20"),
        (96069254, "poseidon_round_keys_output_limb_21"),
        (57701456, "poseidon_round_keys_output_limb_22"),
        (87317035, "poseidon_round_keys_output_limb_23"),
        (71069222, "poseidon_round_keys_output_limb_24"),
        (15362084, "poseidon_round_keys_output_limb_25"),
        (1251686, "poseidon_round_keys_output_limb_26"),
        (61383961, "poseidon_round_keys_output_limb_27"),
        (41881734, "poseidon_round_keys_output_limb_28"),
        (168, "poseidon_round_keys_output_limb_29"),
        (5575208, "combination_limb_0"),
        (85275938, "combination_limb_1"),
        (77047604, "combination_limb_2"),
        (2309211, "combination_limb_3"),
        (40993452, "combination_limb_4"),
        (61821406, "combination_limb_5"),
        (35613761, "combination_limb_6"),
        (127674261, "combination_limb_7"),
        (132409689, "combination_limb_8"),
        (15, "combination_limb_9"),
        (5, "p_coef"),
        (1836878, "combination_limb_0"),
        (19250101, "combination_limb_1"),
        (123204164, "combination_limb_2"),
        (90802953, "combination_limb_3"),
        (13344167, "combination_limb_4"),
        (94265857, "combination_limb_5"),
        (97437854, "combination_limb_6"),
        (126405805, "combination_limb_7"),
        (94599757, "combination_limb_8"),
        (23, "combination_limb_9"),
        (2, "p_coef"),
        (104339820, "combination_limb_0"),
        (45550039, "combination_limb_1"),
        (104590984, "combination_limb_2"),
        (41231498, "combination_limb_3"),
        (81947072, "combination_limb_4"),
        (92489623, "combination_limb_5"),
        (53602251, "combination_limb_6"),
        (20899061, "combination_limb_7"),
        (123123101, "combination_limb_8"),
        (84, "combination_limb_9"),
        (0, "p_coef"),
    "#]]
    .assert_eq(&state.to_string());

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(3),
            [
                const_felt252_expr!(
                    0xcbafc1adb06e95d212132b0c0d0e5646u128,
                    0x208ff902dab0ef051b12fdb03b79a5bu128
                )
                .into(),
                const_felt252_expr!(
                    0x12b2b2f261de39db323c24b6bb80a4e6u128,
                    0x1c2b35cc5f2ffb5c8949be8fdaa0a69u128
                )
                .into(),
                const_felt252_expr!(
                    0x4ea222e01d5fc8d79794cd442dc7e2c7u128,
                    0x6bed17577108d9c3e7248bd95f32a21u128
                )
                .into(),
            ],
        ),
    );
    let expected_output = [
        const_felt252_expr!(
            0x44fa12d484b6715dda64b90368464504u128,
            0x849be1221f8c38be2766fcadb3e4c8u128
        ),
        const_felt252_expr!(
            0xfbd0349285a7795ebd6234122d513069u128,
            0x4af998bff9c606f1b444844f9536292u128
        ),
        const_felt252_expr!(
            0x27192be6b723923be05140f2d45c356eu128,
            0x20aaaee2bc0e29c34a78721e2ca88abu128
        ),
    ];
    for (out, exp_out) in output.2.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }

    let (_, output) = registry.run_air(
        &air_fn,
        (),
        (
            const_expr!(0),
            const_expr!(34),
            [
                const_felt252_expr!(
                    0x932437b264692f535842627ff2a74c19u128,
                    0x1c74c0faeb7590134bad3b2156e51e4u128
                )
                .into(),
                const_felt252_expr!(
                    0xd8cdad6756c0158c4f560628e5035f40u128,
                    0x319010cf1fbc3a6a5cc8d74a7b6e0fbu128
                )
                .into(),
                const_felt252_expr!(
                    0x35c9170386dbaafb0940ba8864dff049u128,
                    0xbcb539f18f2fc93a79d794f43dd10eu128
                )
                .into(),
            ],
        ),
    );
    let expected_output = [
        const_felt252_expr!(
            0xe932a9bf7456d009b1b174f36d558c5u128,
            0xfa8c9b6742b6176139365833d001e3u128
        ),
        const_felt252_expr!(
            0x2d16fba2151e4252a2e2111cde08bfe6u128,
            0x4f04deca4cb7f9f2bd16b1d25b817cau128
        ),
        const_felt252_expr!(
            0x72ab826e9bb5383a8018b59772964892u128,
            0x58dde0a2a785b395ee2dc7b60b79e94u128
        ),
    ];
    for (out, exp_out) in output.2.into_iter().zip(expected_output) {
        assert_eq!(out.calc(), exp_out.calc());
    }
}
