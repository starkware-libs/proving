use std::path::Path;

use air_common::REGISTRY_PROPERTIES_FILE_NAME;
use air_infra::core::air_fn_registry::AirFnStat;
use air_infra::test_utils::{compare_json, compare_registry_jsons};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::PRIME;

use crate::casm::casm_registry::create_casm_registry;

// TODO(AnatG): Remove this
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonComponentStat {
    pub trace_cells_upper_bound: usize,
    pub uses_upper_bound: IndexMap<String, usize>,
    pub steps: usize,
    pub max_num_instances_uses: usize,
    pub max_num_instances_steps: usize,
}

#[test]
fn test_casm_registry() {
    let reg = create_casm_registry();

    // TODO(AnatG): Remove jsons from git.
    compare_registry_jsons(&reg, Path::new("../../outputs/compiled_casm_air/"));

    let stat = reg.collect_stats();
    compare_json(
        &stat,
        &Path::new("../../outputs/compiled_casm_air/").join(REGISTRY_PROPERTIES_FILE_NAME),
    );

    compare_json(
        &IndexMap::from([
            ("keccak".to_string(), get_keccak_stat(&stat)),
            ("ec_op".to_string(), get_ec_op_stat(&stat)),
            ("ecdsa".to_string(), get_ecdsa_stat(&stat)),
        ]),
        Path::new("../../outputs/compiled_casm_air/non_components.json"),
    );
}

// Given a map of values with top level component names and their counts, for 3000 instances of a
// non-component, like keccak, and the statistics computed on all components, returns a
// `NonComponentStat` that contains the upper bounds on trace cells, uses, steps, and max number of
// instances for the given values.
fn get_non_component_stat(
    stat: &IndexMap<String, AirFnStat>,
    values: &IndexMap<String, usize>,
) -> NonComponentStat {
    let mut uses_upper_bound = IndexMap::new();
    let mut trace_cells_upper_bound = 0;
    for (name, cnt) in values.iter() {
        let cells = stat.get(name).unwrap().trace_cells_upper_bound;
        trace_cells_upper_bound += cells * cnt;

        let uses = stat.get(name).unwrap().uses_upper_bound.clone();
        for (use_name, use_bound) in uses.iter() {
            *uses_upper_bound.entry(use_name.clone()).or_default() += *use_bound * cnt;
        }
    }
    for (_, cnt) in uses_upper_bound.iter_mut() {
        *cnt /= 3000;
    }
    trace_cells_upper_bound /= 3000;
    let max_num_instances_uses = PRIME as usize / *uses_upper_bound.values().max().unwrap_or(&1);
    let steps = values
        .iter()
        .filter_map(|(k, v)| if !k.contains("builtin") { Some(v) } else { None })
        .sum::<usize>()
        / 3000;
    let max_num_instances_steps = 2_usize.pow(26) / steps;

    NonComponentStat {
        trace_cells_upper_bound,
        uses_upper_bound,
        steps,
        max_num_instances_uses,
        max_num_instances_steps,
    }
}

fn get_keccak_stat(stat: &IndexMap<String, AirFnStat>) -> NonComponentStat {
    let keccak_3000 = IndexMap::from([
        ("bitwise_builtin".to_string(), 4194304),
        ("mul_mod_builtin".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("poseidon_builtin".to_string(), 0),
        ("range_check_builtin".to_string(), 1048576),
        ("range_check96_builtin".to_string(), 0),
        ("add_mod_builtin".to_string(), 0),
        ("mul_opcode".to_string(), 1651001),
        ("assert_eq_opcode".to_string(), 70300),
        ("jump_opcode_abs".to_string(), 0),
        ("jnz_opcode_non_taken".to_string(), 2029),
        ("ret_opcode".to_string(), 63158),
        ("call_opcode_abs".to_string(), 1),
        ("generic_opcode".to_string(), 0),
        ("assert_eq_opcode_double_deref".to_string(), 12676053),
        ("add_opcode_small".to_string(), 311243),
        ("jump_opcode_rel".to_string(), 0),
        ("assert_eq_opcode_imm".to_string(), 2767034),
        ("jump_opcode_double_deref".to_string(), 0),
        ("add_ap_opcode".to_string(), 50101),
        ("call_opcode_rel_imm".to_string(), 63157),
        ("mul_opcode_small".to_string(), 27099),
        ("add_opcode".to_string(), 1864576),
        ("jnz_opcode_taken".to_string(), 55592),
        ("jump_opcode_rel_imm".to_string(), 7),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("blake_compress_opcode".to_string(), 0),
    ]);
    get_non_component_stat(stat, &keccak_3000)
}

fn get_ec_op_stat(stat: &IndexMap<String, AirFnStat>) -> NonComponentStat {
    let ec_op_3001 = IndexMap::from([
        ("range_check_builtin".to_string(), 16),
        ("mul_mod_builtin".to_string(), 0),
        ("poseidon_builtin".to_string(), 0),
        ("range_check96_builtin".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("add_mod_builtin".to_string(), 0),
        ("bitwise_builtin".to_string(), 0),
        ("jump_opcode_abs".to_string(), 0),
        ("jump_opcode_double_deref".to_string(), 0),
        ("call_opcode_abs".to_string(), 1),
        ("ret_opcode".to_string(), 402288),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("add_ap_opcode".to_string(), 387230),
        ("blake_compress_opcode".to_string(), 0),
        ("add_opcode_small".to_string(), 889522),
        ("assert_eq_opcode_imm".to_string(), 780293),
        ("jnz_opcode_taken".to_string(), 1146967),
        ("assert_eq_opcode_double_deref".to_string(), 1934667),
        ("call_opcode_rel_imm".to_string(), 402287),
        ("jump_opcode_rel_imm".to_string(), 753258),
        ("add_opcode".to_string(), 11983562),
        ("jnz_opcode_non_taken".to_string(), 759282),
        ("generic_opcode".to_string(), 0),
        ("jump_opcode_rel".to_string(), 0),
        ("mul_opcode_small".to_string(), 98),
        ("mul_opcode".to_string(), 6446149),
        ("assert_eq_opcode".to_string(), 2347074),
    ]);

    get_non_component_stat(stat, &ec_op_3001)
}

fn get_ecdsa_stat(stat: &IndexMap<String, AirFnStat>) -> NonComponentStat {
    let ecdsa_3000 = IndexMap::from([
        ("range_check_builtin".to_string(), 16),
        ("range_check96_builtin".to_string(), 262144),
        ("poseidon_builtin".to_string(), 0),
        ("mul_mod_builtin".to_string(), 16384),
        ("bitwise_builtin".to_string(), 0),
        ("pedersen_builtin".to_string(), 512),
        ("add_mod_builtin".to_string(), 0),
        ("generic_opcode".to_string(), 0),
        ("jump_opcode_rel".to_string(), 0),
        ("jump_opcode_abs".to_string(), 0),
        ("mul_opcode_small".to_string(), 98),
        ("qm_31_add_mul_opcode".to_string(), 0),
        ("jnz_opcode_non_taken".to_string(), 768029),
        ("call_opcode_abs".to_string(), 1),
        ("mul_opcode".to_string(), 10737001),
        ("jnz_opcode_taken".to_string(), 3147591),
        ("blake_compress_opcode".to_string(), 0),
        ("jump_opcode_rel_imm".to_string(), 741007),
        ("ret_opcode".to_string(), 933154),
        ("add_ap_opcode".to_string(), 867101),
        ("add_opcode_small".to_string(), 1837238),
        ("assert_eq_opcode".to_string(), 5808292),
        ("jump_opcode_double_deref".to_string(), 0),
        ("assert_eq_opcode_double_deref".to_string(), 4985047),
        ("call_opcode_rel_imm".to_string(), 933153),
        ("add_opcode".to_string(), 19599575),
        ("assert_eq_opcode_imm".to_string(), 2415033),
    ]);

    get_non_component_stat(stat, &ecdsa_3000)
}
