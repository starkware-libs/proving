use super::range_check::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// Macros
use crate::const_expr;
use crate::felt252_expr;

#[test]
fn test_range_check() {
    let deductions = [
        "tmp_0 = ()",
        "tmp_1 = external(Seq_dc507a654de89e80)",
        "tmp_4 = Memory_59f18133215d0936((const_100 + tmp_1))",
        "tmp_4.get_m31(const_0)",
        "tmp_4.get_m31(const_1)",
        "tmp_4.get_m31(const_2)",
        "tmp_4.get_m31(const_3)",
        "tmp_5 = RangeCheck5([state[3]])",
    ];

    let constraints = [
        "tmp_1 = external(Seq_dc507a654de89e80)",
        "Memory_59f18133215d0936([(const_100 + tmp_1)]) == zero_extend([state[0], state[1], state[2], state[3]])",
        "RangeCheck5([state[3]]) == []"
    ];

    let memory = Memory::new_with_data(vec![(
        const_expr!(DUMMY_SEGMENT_START),
        felt252_expr!("value_to_check", (1 << 17), 0),
    )]);

    let rc = RangeCheckBuiltin {
        bits: 32,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);
    let lists = registry.get_compiled_air_fn(&rc);

    assert_eq!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        constraints
    );

    assert_eq!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        deductions
    );
}

fn run_range_check(value: Felt252Expr, bits: usize) {
    let address = DUMMY_SEGMENT_START;
    let memory = Memory::new_with_data(vec![(const_expr!(address), value)]);

    let rc = RangeCheckBuiltin {
        bits,
        memory: memory.clone(),
    };

    let registry = AirFnRegistry::new(&rc);

    registry.run_air_with_row_number(&rc, (), 0);
}

#[test]
fn test_range_check_whole_limbs() {
    run_range_check(felt252_expr!("value_to_check", 1 << 70, 0), 72);
}

#[test]
#[should_panic(expected = "Memory::set() failed")]
fn test_range_check_whole_limbs_fail() {
    run_range_check(felt252_expr!("value_to_check", 1 << 74, 0), 72);
}

// Tests where <bits> is not divisible by 12
#[test]
fn test_range_check_partial_limbs() {
    run_range_check(felt252_expr!("value_to_check", 1 << 127, 0), 128);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck2 on input 4")]
fn test_range_check_partial_limbs_fail() {
    run_range_check(felt252_expr!("value_to_check", 0, 1), 128);
}
