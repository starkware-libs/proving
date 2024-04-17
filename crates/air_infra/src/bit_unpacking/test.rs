use super::super::core::air_fn_registry::*;
use super::super::core::expressions::expr::*;
use super::super::core::expressions::uint16_expr::*;
use super::super::core::prover_types::*;
use super::bit_unpack::BitUnpack;
// Macros
use crate::u16_expr;

#[test]
fn test_bit_unpacking() {
    let func = BitUnpack { n_bits: 4 };
    let (registry, _, lists) = AirFnRegistry::new(&func);

    let constraints = [
        "((state[0] - (state[1] * const_2)) * ((state[0] - (state[1] * const_2)) - const_1))",
        "((state[1] - (state[2] * const_2)) * ((state[1] - (state[2] * const_2)) - const_1))",
        "((state[2] - (state[3] * const_2)) * ((state[2] - (state[3] * const_2)) - const_1))",
        "((state[3] - (state[4] * const_2)) * ((state[3] - (state[4] * const_2)) - const_1))",
        "state[4]",
    ];

    let deductions = [
        "BitUnpack__4_input.as_felt()",
        "tmp_1 = (BitUnpack__4_input >> const_1)",
        "tmp_1.as_felt()",
        "tmp_2 = (tmp_1 >> const_1)",
        "tmp_2.as_felt()",
        "tmp_3 = (tmp_2 >> const_1)",
        "tmp_3.as_felt()",
        "tmp_4 = (tmp_3 >> const_1)",
        "tmp_4.as_felt()",
    ];

    assert!(
        lists
            .constraints
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == constraints
    );
    assert!(
        lists
            .deductions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            == deductions
    );

    let (state, output) = registry.run_air(&func, u16_expr!("x", 10));
    assert!(state.calc() == ["10", "5", "2", "1", "0"]);
    assert!(
        output.iter().map(|x| x.calc()).collect::<Vec<String>>()
            == ["false", "true", "false", "true"]
    );
}
