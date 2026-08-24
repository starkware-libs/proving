use circuits::context::{TraceContext, Var};
use circuits::ivalue::qm31_from_u32s;
use circuits::wrappers::M31Wrapper;

use crate::statement::{AuxData, CasmState, PubMemoryAddress, SegmentRange, public_logup_sum};

/// Returns a constant base-field variable holding `value`.
fn m31_const(context: &mut TraceContext, value: u32) -> M31Wrapper<Var> {
    M31Wrapper::const_m31(context, value.into())
}

#[test]
fn test_public_logup_sum() {
    let mut context = TraceContext::default();

    // Create dummy lookup elements (interaction elements)
    // Matching LookupElementsDummyImpl::dummy() from Cairo1:
    // z = qm31_const<1, 2, 3, 4>(), alpha = One::one()
    let interaction_z = context.constant(qm31_from_u32s(1, 2, 3, 4));
    let interaction_alpha = context.constant(qm31_from_u32s(4, 3, 2, 1));
    let interaction_elements = [interaction_z, interaction_alpha];

    // Create initial state: CasmState uses { pc, ap, fp } order
    // Cairo1 has: pc=1, ap=1336, fp=1336
    let initial_state = CasmState {
        pc: m31_const(&mut context, 1),
        ap: m31_const(&mut context, 1336),
        fp: m31_const(&mut context, 1336),
    };

    // Create final state: CasmState uses { pc, ap, fp } order
    // Cairo1 has: pc=5, ap=2520, fp=1336
    let final_state = CasmState {
        pc: m31_const(&mut context, 5),
        ap: m31_const(&mut context, 2520),
        fp: m31_const(&mut context, 1336),
    };

    // Create segment ranges matching the Cairo1 test
    // The 11 segments are: output, pedersen, range_check_128, ecdsa, bitwise, ec_op, keccak,
    // poseidon, range_check_96, add_mod, mul_mod
    let segment_ranges = [
        // output: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // pedersen: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // range_check_128: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // ecdsa: id=5, value=0
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
        },
        // bitwise: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // ec_op: id=5, value=0
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
        },
        // keccak: id=5, value=0
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 5),
                value: m31_const(&mut context, 0),
            },
        },
        // poseidon: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // range_check_96: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // add_mod: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
        // mul_mod: id=228, value=2520
        SegmentRange {
            start: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
            end: PubMemoryAddress {
                id: m31_const(&mut context, 228),
                value: m31_const(&mut context, 2520),
            },
        },
    ];

    // Create safe_call_ids: [227, 5]
    let safe_call_id_0 = m31_const(&mut context, 227);
    let safe_call_id_1 = m31_const(&mut context, 5);
    let safe_call_ids = [safe_call_id_0, safe_call_id_1];

    // Create empty output vector
    let output_ids: Vec<M31Wrapper<Var>> = vec![];
    let outputs = [];

    // The `ret opcode` program split into 9bit limbs.
    let program = [
        [
            511, 447, 511, 47, 0, 60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 192, 0, 48, 0, 36, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            511, 447, 511, 47, 0, 60, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 448, 511, 111, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 143, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 175, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 207, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 239, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 271, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 303, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 335, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 367, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 399, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 448, 511, 431, 511, 83, 288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            510, 447, 511, 495, 511, 91, 130, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
    ]
    .iter()
    .map(|limbs| limbs.map(|limb| M31Wrapper::const_m31(&mut context, limb.into())))
    .collect::<Vec<_>>();

    let program_ids =
        (0..program.len()).map(|id| m31_const(&mut context, id as u32)).collect::<Vec<_>>();

    let aux_data = AuxData {
        initial_state,
        final_state,
        segment_ranges,
        safe_call_ids,
        output_ids,
        program_ids,
        // Not used by `public_logup_sum`.
        component_log_sizes: vec![],
    };

    // Call public_logup_sum
    let result =
        public_logup_sum(&mut context, &aux_data, &program[..], &outputs[..], interaction_elements);

    // The result should be a valid variable
    let result_value = context.get(result);
    context.mark_as_unused(result);

    let expected = qm31_from_u32s(908842852, 42171643, 313383432, 1019452808);

    assert_eq!(result_value, expected);

    // Validate the circuit
    let context = context.finalize(true);
    context.circuit().check_yields();
    context.validate_circuit();
}
