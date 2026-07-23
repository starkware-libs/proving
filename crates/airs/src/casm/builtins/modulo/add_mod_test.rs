use std::array::from_fn;

use air_infra::core::Felt;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::core::state::State;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::add_mod::*;
use super::mod_utils::*;

type BigInt = [u128; MOD_BUILTIN_N_WORDS];

pub const VALID_ABCP_BANK: [(BigInt, [[BigInt; 3]; 5]); 2] = [
    (
        [
            50194602145281500560007247715,
            76868189491771433761047631975,
            76613164022382921729294390885,
            46298721445880844022721490358,
        ],
        [
            [
                [
                    29091876654323145151174848554,
                    26007157788849058493269479829,
                    38481985774736263908649278847,
                    57744057286257910072133617967,
                ],
                [
                    30637063040695702493959780639,
                    49380517041604201135167969030,
                    29641425794583440171879443246,
                    19614206973152481265685278126,
                ],
                [
                    9534337549737347085127381478,
                    77747647852946163460933767220,
                    70738410061201119944778281543,
                    31059542813529547315097405734,
                ],
            ],
            [
                [
                    25364755434755755068608619509,
                    46375831507992734287877107353,
                    51670430756152926393861588720,
                    52429320694060955929043194660,
                ],
                [
                    78945914572755310267435273962,
                    27712413850847538634239209051,
                    65428338942922928372768070484,
                    28052795324322022483964753387,
                ],
                [
                    54116067862229564776036645756,
                    76448218381333176754612634765,
                    40485605676692933037335268318,
                    34183394572502134390286457689,
                ],
            ],
            [
                [
                    35968495518893490208806073068,
                    49102443134823650461092222214,
                    35464705345053458822988979448,
                    65168685965630356325848133284,
                ],
                [
                    18826251302685506831345344084,
                    77274307093685476183699352459,
                    15618203299442742321497109198,
                    35250365537155797686444094446,
                ],
                [
                    4600144676297496480144169437,
                    49508560736737692883743942698,
                    53697907136377617008735648097,
                    54120330056905309989570737371,
                ],
            ],
            [
                [
                    60883563942469309009486599862,
                    60066628311203458389370527088,
                    7449274889379704727124781271,
                    11189203920044496436990168234,
                ],
                [
                    50836092169352032718968957818,
                    19767672974274733933438133414,
                    45660900186427624193304759903,
                    43099436324578250534149449064,
                ],
                [
                    32491493597557004134911607344,
                    606138771213854729264710167,
                    53110175075807328920429541175,
                    54288640244622746971139617298,
                ],
            ],
            [
                [
                    60883563942469309009486599862,
                    60066628311203458389370527088,
                    7449274889379704727124781271,
                    11189203920044496436990168234,
                ],
                [
                    50836092169352032718968957818,
                    19767672974274733933438133414,
                    45660900186427624193304759903,
                    43099436324578250534149449064,
                ],
                [
                    61525053966539841168448309965,
                    2966111793706758561761028527,
                    55725173567688744784679100625,
                    7989918798741902948418126939,
                ],
            ],
        ],
    ),
    (
        [
            47215045371565379164322085426,
            44041118390009378566376440305,
            28253136013572507312084647372,
            27591888424968618459251648517,
        ],
        [
            [
                [
                    69926110382496690563359520275,
                    9308974168209872844197265502,
                    64424777588392514699752119546,
                    5394210003713718564961882557,
                ],
                [
                    2845066666780114342938595450,
                    29780086105520932566160438017,
                    75213321032037721471593296313,
                    6611815142597078571092015255,
                ],
                [
                    72771177049276804906298115725,
                    39089060273730805410357703519,
                    60409936106165898577801465523,
                    12006025146310797136053897813,
                ],
            ],
            [
                [
                    76598284407932491796650762642,
                    66981425700865223582276481415,
                    25416153705972621336493592301,
                    44488314473329323761662510398,
                ],
                [
                    28309971518945781801054774815,
                    35138656155715921972802013577,
                    33567915509971885704499946320,
                    56383220350781727457838908033,
                ],
                [
                    57693210555312894433383452031,
                    58078963466571766988702054687,
                    30730933202371999728908891249,
                    73279646399142432760249769914,
                ],
            ],
            [
                [
                    12227190523074426419090527623,
                    46712652064633827086171544646,
                    46055537532622719551869569698,
                    7385742337844151409533505471,
                ],
                [
                    41502782606238570599766817469,
                    57894090910609718724726438504,
                    4497518638534961828729279062,
                    78278135709032233327300018245,
                ],
                [
                    6514927757747617854535259666,
                    60565624585234167244521542845,
                    22299920157585174068514201388,
                    58071989621907766277581875199,
                ],
            ],
            [
                [
                    28867853495640247994985463948,
                    18585628989548478401878201783,
                    77006817547779051930262317139,
                    43535633033191100986874357118,
                ],
                [
                    52618273337008354256391659545,
                    23134206742823158282286068248,
                    8015530501547549593174893937,
                    55120551901367333424815150534,
                ],
                [
                    34271081461083223087055038067,
                    76906879856626595711331780062,
                    56769212035754094211352563703,
                    71064296509589815952437859135,
                ],
            ],
            [
                [
                    44390217778450106642788121428,
                    41707332170547363733210108810,
                    29530285266187886030904079407,
                    6804191761749355142818135605,
                ],
                [
                    44618248458250066350674949180,
                    71517080126292087066788763066,
                    29803266097382181021625108294,
                    35684685716400274220441588037,
                ],
                [
                    9780303722435835399919120272,
                    33996249782575113206454921541,
                    59333551363570067052529187702,
                    42488877478149629363259723642,
                ],
            ],
        ],
    ),
];

fn data_unravel_2d(data: [[FeltExpr; MOD_BUILTIN_N_WORDS]; 3]) -> Vec<FeltExpr> {
    data.iter().flatten().cloned().collect::<Vec<_>>()
}

fn data_unravel_2d_252(data: [[Felt252Expr; MOD_BUILTIN_N_WORDS]; 3]) -> Vec<Felt252Expr> {
    data.iter().flatten().cloned().collect::<Vec<_>>()
}

fn run_add_mod_builtin(instances: Vec<AddModInstance>) -> Vec<State> {
    let segment_start = 200;
    let mut memory_addr_to_vals = vec![];
    for (ind, instance) in instances.iter().enumerate() {
        let offsets_ptr_val = instance.offsets_ptr;
        let offsets_ptr_m31 = const_expr!(offsets_ptr_val);
        let values_ptr_val = instance.values_ptr;
        let values_ptr_m31 = const_expr!(values_ptr_val);

        let offsets_addr: [FeltExpr; 3] =
            core::array::from_fn(|j| const_expr!(j as u32) + offsets_ptr_m31.clone());
        let offsets_val_int: [usize; 3] =
            core::array::from_fn(|j| MOD_BUILTIN_N_WORDS * j + 3 * MOD_BUILTIN_N_WORDS * ind);
        let offsets_vals: [Felt252Expr; 3] =
            core::array::from_fn(|j| const_felt252_expr!(offsets_val_int[j] as u128, 0));

        let vars_addr: [[FeltExpr; MOD_BUILTIN_N_WORDS]; 3] = core::array::from_fn(|j| {
            core::array::from_fn(|k| {
                const_expr!((k + offsets_val_int[j]) as u32) + values_ptr_m31.clone()
            })
        });

        let vars_vals: [[Felt252Expr; 4]; 3] =
            from_fn(|i| from_fn(|j| const_felt252_expr!(instance.abc[i][j], 0)));
        let p_val: [Felt252Expr; 4] = from_fn(|j| const_felt252_expr!(instance.p[j], 0));

        memory_addr_to_vals.extend(vec![
            (
                const_expr!(segment_start + 4 + 7 * ind as u32),
                const_felt252_expr!(values_ptr_val as u128, 0),
            ),
            (
                const_expr!(segment_start + 5 + 7 * ind as u32),
                const_felt252_expr!(offsets_ptr_val as u128, 0),
            ),
            (
                const_expr!(segment_start + 6 + 7 * ind as u32),
                const_felt252_expr!(instance.n as u128, 0),
            ),
        ]);
        memory_addr_to_vals.extend(
            (0..4)
                .map(|i| const_expr!(segment_start + i + 7 * ind as u32))
                .zip(p_val.iter().cloned()),
        );

        memory_addr_to_vals.extend(offsets_addr.into_iter().zip(offsets_vals));
        memory_addr_to_vals
            .extend(data_unravel_2d(vars_addr).into_iter().zip(data_unravel_2d_252(vars_vals)));
    }
    let memory = Felt252IdMemory::new_with_data(memory_addr_to_vals);
    let add_mod = AddModBuiltin { memory };

    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::AddModBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&add_mod);

    let mut state_per_instance = vec![];
    for row in 0..instances.len() {
        let (curr_state, ..) = registry.run_air_with_row_number(&add_mod, (), (), row);
        state_per_instance.push(curr_state);
    }

    state_per_instance
}

fn sequence_from_bank(
    p_index: usize,
    start: usize,
    end: usize,
    offsets_ptr_val0: u32,
    values_ptr_val0: u32,
) -> Vec<AddModInstance> {
    (start..end)
        .map(|i| AddModInstance {
            p: VALID_ABCP_BANK[p_index].0,
            abc: VALID_ABCP_BANK[p_index].1[i],
            values_ptr: values_ptr_val0,
            offsets_ptr: offsets_ptr_val0 + (3 * i) as u32,
            n: end - i,
        })
        .collect::<Vec<AddModInstance>>()
}

struct AddModInstance {
    p: [u128; MOD_BUILTIN_N_WORDS],
    abc: [[u128; MOD_BUILTIN_N_WORDS]; 3],
    values_ptr: u32,
    offsets_ptr: u32,
    n: usize,
}

impl AddModInstance {
    fn distort(&mut self, target: &str, distortion: i32, nword: usize) {
        match target {
            "p" => {
                self.p[nword] = (self.p[nword] as i32 + distortion) as u128;
            }
            "a" | "b" | "c" => {
                let index = match target {
                    "a" => 0,
                    "b" => 1,
                    "c" => 2,
                    _ => panic!("Invalid target"),
                };
                self.abc[index][nword] = (self.abc[index][nword] as i32 + distortion) as u128;
            }
            "values_ptr" => self.values_ptr = (self.values_ptr as i32 + distortion) as u32,
            "offsets_ptr" => self.offsets_ptr = (self.offsets_ptr as i32 + distortion) as u32,
            "n" => self.n = (self.n as i32 + distortion) as usize,
            _ => panic!("Invalid target"),
        }
    }
}

#[test]
fn test_add_mod_builtin_on_abcp_bank() {
    let mut instances = sequence_from_bank(0, 1, VALID_ABCP_BANK[0].1.len(), 600, 1000);
    instances.extend(sequence_from_bank(1, 0, 4, 1400, 1900));
    run_add_mod_builtin(instances);
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_add_mod_builtin_on_distorted_n() {
    let mut instances = sequence_from_bank(0, 2, VALID_ABCP_BANK[0].1.len(), 600, 1000);
    for instance in instances.iter_mut() {
        instance.distort("n", 1, 0);
    }
    instances.extend(sequence_from_bank(1, 0, 4, 1400, 1900));
    run_add_mod_builtin(instances);
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_add_mod_builtin_on_distorted_abcp() {
    let mut instances = sequence_from_bank(0, 1, 2, 600, 1000);
    instances[0].distort("b", 1, 2);
    run_add_mod_builtin(instances);
}

#[test]
fn test_add_mod_builtin_state() {
    let instances = sequence_from_bank(0, 2, 3, 550, 1050);
    let state = run_add_mod_builtin(instances);

    expect![[r#"
        (1, "is_instance_0"),
        (3, "p0_id"),
        (355, "p0_limb_0"),
        (283, "p0_limb_1"),
        (76, "p0_limb_2"),
        (423, "p0_limb_3"),
        (228, "p0_limb_4"),
        (276, "p0_limb_5"),
        (151, "p0_limb_6"),
        (229, "p0_limb_7"),
        (0, "p0_limb_8"),
        (280, "p0_limb_9"),
        (40, "p0_limb_10"),
        (4, "p1_id"),
        (103, "p1_limb_0"),
        (108, "p1_limb_1"),
        (250, "p1_limb_2"),
        (425, "p1_limb_3"),
        (197, "p1_limb_4"),
        (362, "p1_limb_5"),
        (102, "p1_limb_6"),
        (140, "p1_limb_7"),
        (480, "p1_limb_8"),
        (47, "p1_limb_9"),
        (62, "p1_limb_10"),
        (5, "p2_id"),
        (101, "p2_limb_0"),
        (363, "p2_limb_1"),
        (93, "p2_limb_2"),
        (99, "p2_limb_3"),
        (361, "p2_limb_4"),
        (337, "p2_limb_5"),
        (207, "p2_limb_6"),
        (271, "p2_limb_7"),
        (236, "p2_limb_8"),
        (454, "p2_limb_9"),
        (61, "p2_limb_10"),
        (6, "p3_id"),
        (438, "p3_limb_0"),
        (406, "p3_limb_1"),
        (18, "p3_limb_2"),
        (315, "p3_limb_3"),
        (433, "p3_limb_4"),
        (87, "p3_limb_5"),
        (252, "p3_limb_6"),
        (317, "p3_limb_7"),
        (359, "p3_limb_8"),
        (204, "p3_limb_9"),
        (37, "p3_limb_10"),
        (0, "values_ptr_id"),
        (26, "values_ptr_limb_0"),
        (2, "values_ptr_limb_1"),
        (0, "values_ptr_limb_2"),
        (0, "values_ptr_limb_3"),
        (0, "partial_limb_msb"),
        (1, "offsets_ptr_id"),
        (44, "offsets_ptr_limb_0"),
        (1, "offsets_ptr_limb_1"),
        (0, "offsets_ptr_limb_2"),
        (0, "offsets_ptr_limb_3"),
        (0, "partial_limb_msb"),
        (1, "offsets_ptr_prev_id"),
        (44, "offsets_ptr_prev_limb_0"),
        (1, "offsets_ptr_prev_limb_1"),
        (0, "offsets_ptr_prev_limb_2"),
        (0, "offsets_ptr_prev_limb_3"),
        (0, "partial_limb_msb"),
        (2, "n_id"),
        (1, "n_limb_0"),
        (0, "n_limb_1"),
        (0, "n_limb_2"),
        (0, "n_limb_3"),
        (0, "partial_limb_msb"),
        (2, "n_prev_id"),
        (1, "n_prev_limb_0"),
        (0, "n_prev_limb_1"),
        (0, "n_prev_limb_2"),
        (0, "n_prev_limb_3"),
        (0, "partial_limb_msb"),
        (0, "values_ptr_prev_id"),
        (3, "p_prev0_id"),
        (4, "p_prev1_id"),
        (5, "p_prev2_id"),
        (6, "p_prev3_id"),
        (7, "offsets_a_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (0, "offsets_a_limb_0"),
        (0, "offsets_a_limb_1"),
        (0, "offsets_a_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (8, "offsets_b_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (4, "offsets_b_limb_0"),
        (0, "offsets_b_limb_1"),
        (0, "offsets_b_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (9, "offsets_c_id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (8, "offsets_c_limb_0"),
        (0, "offsets_c_limb_1"),
        (0, "offsets_c_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
        (10, "a0_id"),
        (236, "a0_limb_0"),
        (121, "a0_limb_1"),
        (377, "a0_limb_2"),
        (226, "a0_limb_3"),
        (201, "a0_limb_4"),
        (59, "a0_limb_5"),
        (134, "a0_limb_6"),
        (98, "a0_limb_7"),
        (113, "a0_limb_8"),
        (28, "a0_limb_9"),
        (29, "a0_limb_10"),
        (11, "a1_id"),
        (262, "a1_limb_0"),
        (138, "a1_limb_1"),
        (237, "a1_limb_2"),
        (109, "a1_limb_3"),
        (440, "a1_limb_4"),
        (221, "a1_limb_5"),
        (57, "a1_limb_6"),
        (400, "a1_limb_7"),
        (150, "a1_limb_8"),
        (340, "a1_limb_9"),
        (39, "a1_limb_10"),
        (12, "a2_id"),
        (248, "a2_limb_0"),
        (302, "a2_limb_1"),
        (432, "a2_limb_2"),
        (75, "a2_limb_3"),
        (153, "a2_limb_4"),
        (133, "a2_limb_5"),
        (85, "a2_limb_6"),
        (242, "a2_limb_7"),
        (439, "a2_limb_8"),
        (331, "a2_limb_9"),
        (28, "a2_limb_10"),
        (13, "a3_id"),
        (164, "a3_limb_0"),
        (151, "a3_limb_1"),
        (139, "a3_limb_2"),
        (10, "a3_limb_3"),
        (301, "a3_limb_4"),
        (233, "a3_limb_5"),
        (100, "a3_limb_6"),
        (18, "a3_limb_7"),
        (70, "a3_limb_8"),
        (329, "a3_limb_9"),
        (52, "a3_limb_10"),
        (14, "b0_id"),
        (84, "b0_limb_0"),
        (175, "b0_limb_1"),
        (402, "b0_limb_2"),
        (233, "b0_limb_3"),
        (204, "b0_limb_4"),
        (20, "b0_limb_5"),
        (109, "b0_limb_6"),
        (400, "b0_limb_7"),
        (181, "b0_limb_8"),
        (106, "b0_limb_9"),
        (15, "b0_limb_10"),
        (15, "b1_id"),
        (395, "b1_limb_0"),
        (501, "b1_limb_1"),
        (165, "b1_limb_2"),
        (350, "b1_limb_3"),
        (402, "b1_limb_4"),
        (119, "b1_limb_5"),
        (424, "b1_limb_6"),
        (10, "b1_limb_7"),
        (463, "b1_limb_8"),
        (215, "b1_limb_9"),
        (62, "b1_limb_10"),
        (16, "b2_id"),
        (206, "b2_limb_0"),
        (249, "b2_limb_1"),
        (290, "b2_limb_2"),
        (511, "b2_limb_3"),
        (395, "b2_limb_4"),
        (458, "b2_limb_5"),
        (387, "b2_limb_6"),
        (98, "b2_limb_7"),
        (275, "b2_limb_8"),
        (315, "b2_limb_9"),
        (12, "b2_limb_10"),
        (17, "b3_id"),
        (494, "b3_limb_0"),
        (505, "b3_limb_1"),
        (141, "b3_limb_2"),
        (227, "b3_limb_3"),
        (184, "b3_limb_4"),
        (89, "b3_limb_5"),
        (322, "b3_limb_6"),
        (129, "b3_limb_7"),
        (107, "b3_limb_8"),
        (243, "b3_limb_9"),
        (28, "b3_limb_10"),
        (18, "c0_id"),
        (477, "c0_limb_0"),
        (12, "c0_limb_1"),
        (191, "c0_limb_2"),
        (37, "c0_limb_3"),
        (177, "c0_limb_4"),
        (315, "c0_limb_5"),
        (91, "c0_limb_6"),
        (269, "c0_limb_7"),
        (294, "c0_limb_8"),
        (366, "c0_limb_9"),
        (3, "c0_limb_10"),
        (19, "c1_id"),
        (42, "c1_limb_0"),
        (20, "c1_limb_1"),
        (153, "c1_limb_2"),
        (34, "c1_limb_3"),
        (133, "c1_limb_4"),
        (491, "c1_limb_5"),
        (378, "c1_limb_6"),
        (270, "c1_limb_7"),
        (133, "c1_limb_8"),
        (508, "c1_limb_9"),
        (39, "c1_limb_10"),
        (20, "c2_id"),
        (353, "c2_limb_0"),
        (188, "c2_limb_1"),
        (117, "c2_limb_2"),
        (488, "c2_limb_3"),
        (187, "c2_limb_4"),
        (254, "c2_limb_5"),
        (265, "c2_limb_6"),
        (69, "c2_limb_7"),
        (478, "c2_limb_8"),
        (192, "c2_limb_9"),
        (43, "c2_limb_10"),
        (21, "c3_id"),
        (219, "c3_limb_0"),
        (250, "c3_limb_1"),
        (262, "c3_limb_2"),
        (434, "c3_limb_3"),
        (51, "c3_limb_4"),
        (235, "c3_limb_5"),
        (170, "c3_limb_6"),
        (342, "c3_limb_7"),
        (329, "c3_limb_8"),
        (367, "c3_limb_9"),
        (43, "c3_limb_10"),
        (1, "sub_p_bit"),
        (1, "carry_0"),
        (2147483646, "carry_1"),
        (0, "carry_2"),
        (1, "carry_3"),
        (0, "carry_4"),
        (0, "carry_5"),
        (0, "carry_6"),
        (0, "carry_7"),
        (0, "carry_8"),
        (0, "carry_9"),
        (2147483646, "carry_10"),
        (0, "carry_11"),
        (0, "carry_12"),
        (2147483646, "carry_13"),
    "#]]
    .assert_eq(&state[0].to_string());
}
