#[generic_tests::define]
mod tests {
    use std::array::from_fn;

    use air_infra::core::air_fn_registry::*;
    use air_infra::core::expressions::felt_expr::*;
    use air_infra::core::expressions::felt252_expr::*;
    use air_infra::core::public_params::PublicParam;
    use air_infra::core::variables::*;
    use air_infra::core::*;
    use air_infra::felt252_id_memory::memory::*;
    use air_infra::{const_expr, const_felt252_expr, const_felt252_expr_from_felt252};

    use crate::casm::builtins::ec_utils::utils::*;
    use crate::casm::builtins::pedersen::partial_ec_mul::*;
    use crate::casm::builtins::pedersen::pedersen_aggregator::*;
    use crate::casm::builtins::pedersen::pedersen_builtin::*;

    fn pack_to_limbs<const NUM_WINDOWS: usize>(mut value: u64) -> PackedECMultiplier<NUM_WINDOWS> {
        let window_bits: usize = 252 / NUM_WINDOWS;
        let mask: u64 = (1 << window_bits) - 1;
        from_fn(|_| {
            let double_limb = const_expr!(
                TryInto::<u32>::try_into(value & mask)
                    .expect("After masking the value should be small")
            );
            value >>= window_bits;
            double_limb
        })
    }

    #[test]
    fn test_partial_mul<const NUM_WINDOWS: usize>() {
        let window_bits = 252 / NUM_WINDOWS;
        let air_fn = &PartialECMul::<NUM_WINDOWS>::new();
        let (registry, _) = AirFnRegistry::new(air_fn);

        let call_id = const_expr!(0);
        // Round number that indicates the start of the P_2 block
        let round_num = const_expr!(NUM_WINDOWS);

        // The coordinates of the P_1 Pedersen point
        // let sum_0_pt = ec_mul(&P_SHIFT, 2 * NUM_WINDOWS + 1);
        let [p1_x, p1_y] =
            [const_felt252_expr_from_felt252!(P_1.x), const_felt252_expr_from_felt252!(P_1.y)];

        // The coordinates of 123*P_2 + P_1 - 2*P_shift
        let neg_p_shift = ec_neg(&P_SHIFT);
        let result = ec_add(&ec_add_mul(&P_1, &P_2, 123), &neg_p_shift);
        let [result_x, result_y] = [
            const_felt252_expr_from_felt252!(result.x),
            const_felt252_expr_from_felt252!(result.y),
        ];

        let multiplier = (7 << window_bits) + 123;
        let (state, output) = registry.run_air(
            air_fn,
            (),
            (call_id.clone(), round_num, (pack_to_limbs(multiplier), [p1_x, p1_y])),
        );
        assert_eq!(output.0.calc(), call_id.calc());
        assert_eq!(output.1.calc(), const_expr!(NUM_WINDOWS + 1).calc());

        let expected_new_multiplier = pack_to_limbs::<NUM_WINDOWS>(multiplier >> window_bits);
        for (output_elem, expected_elem) in output.2.0.iter().zip(expected_new_multiplier.iter()) {
            assert_eq!(output_elem.calc(), expected_elem.calc());
        }
        assert_eq!(output.2.1[0].calc(), result_x.calc());
        assert_eq!(output.2.1[1].calc(), result_y.calc());
        let expected_trace_len = match NUM_WINDOWS {
            14 => 297,
            28 => 311,
            _ => panic!("Unsupported NUM_WINDOWS val {NUM_WINDOWS}"),
        };
        assert_eq!(state.get_felts().len(), expected_trace_len);
    }

    #[test]
    fn test_pedersen_0<const NUM_WINDOWS: usize>() {
        let segment_start = 500;

        let memory = Felt252IdMemory::new_with_data(vec![
            (const_expr!(segment_start), const_felt252_expr!(0, 0)),
            (const_expr!(segment_start + 1), const_felt252_expr!(0, 0)),
            (const_expr!(segment_start + 2), const_felt252_expr_from_felt252!(P_SHIFT.x)),
        ]);

        let pedersen = PedersenBuiltin::<NUM_WINDOWS> { memory: memory.clone() };
        let mut registry = AirFnRegistry::new_empty();
        registry
            .public_params
            .set(PublicParam::PedersenBuiltinSegmentStart, Felt::from(segment_start));
        registry.add_entry(&pedersen);

        let (state, _) = registry.run_air_with_row_number(&pedersen, (), (), 0);
        assert_eq!(state.get_felts().len(), 3);

        let (state, _) = registry.run_air(
            &PedersenAggregator::<NUM_WINDOWS>::new(memory),
            (),
            (
                [CasmId::new(const_expr!(0), "a"), CasmId::new(const_expr!(0), "b")],
                CasmId::new(const_expr!(1), "output"),
            ),
        );
        let expected_trace_len = match NUM_WINDOWS {
            14 => 206,
            28 => 234,
            _ => panic!("Unsupported NUM_WINDOWS val {NUM_WINDOWS}"),
        };
        assert_eq!(state.get_felts().len(), expected_trace_len);
    }

    #[test]
    fn test_pedersen_random<const NUM_WINDOWS: usize>() {
        let segment_start = 500;

        let memory = Felt252IdMemory::new_with_data(vec![
            (
                const_expr!(segment_start),
                const_felt252_expr!(
                    0x7e3dcae2971a7b5e7c51bb79d6e1ef97,
                    0x2b4ad973e17143178840b3ddac9c771
                ),
            ),
            (
                const_expr!(segment_start + 1),
                const_felt252_expr!(
                    0xcb577c90056935d2608096e848573e39,
                    0x4678487e5de069f1b159422e625fb6b
                ),
            ),
            (
                const_expr!(segment_start + 2),
                const_felt252_expr!(
                    0xf03ff7ee2bec85660ef9431bb21b0dfd,
                    0x3b68ae30d53cd5d25410b744c249e4f
                ),
            ),
        ]);

        let pedersen = PedersenBuiltin::<NUM_WINDOWS> { memory };
        let mut registry = AirFnRegistry::new_empty();
        registry
            .public_params
            .set(PublicParam::PedersenBuiltinSegmentStart, Felt::from(segment_start));
        registry.add_entry(&pedersen);

        registry.run_air_with_row_number(&pedersen, (), (), 0);
    }

    #[test]
    #[should_panic(expected = "Added incorrect constraint")]
    fn test_pedersen_unreduced<const NUM_WINDOWS: usize>() {
        let segment_start = 500;

        // Make sure the Pedersen builtin doesn't reduce inputs modulo P252
        let memory = Felt252IdMemory::new_with_data(vec![
            // a = P
            (
                const_expr!(segment_start),
                const_felt252_expr!(0x1, 0x8000000000000110000000000000000),
            ),
            // b = 0
            (const_expr!(segment_start + 1), const_felt252_expr!(0, 0)),
            // result = Pedersen(0,0)
            (const_expr!(segment_start + 2), const_felt252_expr_from_felt252!(P_SHIFT.x)),
        ]);

        let pedersen = PedersenBuiltin::<NUM_WINDOWS> { memory };
        let mut registry = AirFnRegistry::new_empty();
        registry
            .public_params
            .set(PublicParam::PedersenBuiltinSegmentStart, Felt::from(segment_start));
        registry.add_entry(&pedersen);

        registry.run_air_with_row_number(&pedersen, (), (), 0);
    }

    #[test]
    #[should_panic(expected = "Added incorrect constraint")]
    fn test_pedersen_unreduced2<const NUM_WINDOWS: usize>() {
        let segment_start = 500;

        // Make sure the Pedersen builtin doesn't process >P252 inputs as-is.
        let memory = Felt252IdMemory::new_with_data(vec![
            // a = P
            (
                const_expr!(segment_start),
                const_felt252_expr!(0x1, 0x8000000000000110000000000000000),
            ),
            // b = 0
            (const_expr!(segment_start + 1), const_felt252_expr!(0, 0)),
            // result = Pedersen(P,0)
            (
                const_expr!(segment_start + 2),
                const_felt252_expr!(
                    0x70f3d27254d3082c700711659b6e7081,
                    0x2703e5efb0ce387bc1ea350e6abcb36
                ),
            ),
        ]);

        let pedersen = PedersenBuiltin::<NUM_WINDOWS> { memory };
        let mut registry = AirFnRegistry::new_empty();
        registry
            .public_params
            .set(PublicParam::PedersenBuiltinSegmentStart, Felt::from(segment_start));
        registry.add_entry(&pedersen);

        registry.run_air_with_row_number(&pedersen, (), (), 0);
    }

    #[instantiate_tests(<28>)]
    mod small_window {}

    #[instantiate_tests(<14>)]
    mod large_window {}
}
