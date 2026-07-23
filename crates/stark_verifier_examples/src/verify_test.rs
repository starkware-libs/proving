use circuit_serialize::serialize::CircuitSerialize;
use circuits::context::{Context, TraceContext};
use circuits::ivalue::NoValue;
use circuits::ops::Guess;
use circuits_stark_verifier::fri_proof::compute_all_fold_steps;
use circuits_stark_verifier::proof::{ProofConfig, ProofInfo, empty_proof};
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use circuits_stark_verifier::verify::verify;
use num_traits::One;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;

use crate::simple_air::{create_proof, create_proof_with_fold_step};
use crate::simple_statement::{
    PREPROCESSED_COLUMN_LOG_SIZES, SimpleStatement, simple_statement_components,
};

enum ProofModifier {
    /// Keep the proof unchanged.
    None,
    /// Modify an element of the authentication path for one of the traces.
    WrongTraceAuthPath,
    /// Modify an element of the first layer Merkle authentication path (decommitment).
    WrongFriAuthPath,
    /// Modify the queried coset value at the last inner layer of FRI.
    WrongFriWitness,
}

#[rstest]
#[case::success(ProofModifier::None)]
#[case::wrong_trace_auth_path(ProofModifier::WrongTraceAuthPath)]
#[case::wrong_fri_auth_path(ProofModifier::WrongFriAuthPath)]
#[case::wrong_fri_witness(ProofModifier::WrongFriWitness)]
fn test_verify(#[case] proof_modifier: ProofModifier) {
    let (_components, claim, pcs_config, mut proof, interaction_pow_nonce, channel_salt) =
        create_proof();

    let components = simple_statement_components::<NoValue>();
    let config = ProofConfig::new(&components, PREPROCESSED_COLUMN_LOG_SIZES.len(), &pcs_config, 8);
    // Create a NoValue version.
    let novalue_circuit = {
        let empty_proof = empty_proof(&config);
        let mut novalue_context = Context::<NoValue>::default();
        let proof_vars = empty_proof.guess(&mut novalue_context);
        let statement = SimpleStatement::new(&mut novalue_context);
        verify(&mut novalue_context, &proof_vars, &config, &statement);
        novalue_context.finalize(false).context.circuit
    };

    match proof_modifier {
        ProofModifier::None => {}
        ProofModifier::WrongTraceAuthPath => {
            let first_query = proof.aux.unsorted_query_locations[0];
            // `trace_decommitment[1]` refers to the main trace.
            let first_layer_values = &mut proof.aux.trace_decommitment[1].all_node_values[0];
            let value: &mut Blake2sHash = first_layer_values.get_mut(&(first_query ^ 1)).unwrap();
            value.0[0] ^= 1;
        }
        ProofModifier::WrongFriAuthPath => {
            let first_query = proof.aux.unsorted_query_locations[0];
            let first_layer_values = &mut proof.aux.fri.first_layer.decommitment.all_node_values[1];
            let value: &mut Blake2sHash =
                first_layer_values.get_mut(&((first_query >> 1) ^ 1)).unwrap();
            value.0[0] ^= 1;
        }
        ProofModifier::WrongFriWitness => {
            let values = &mut proof.aux.fri.inner_layers.last_mut().unwrap().all_values[0];
            for (_, value) in values.iter_mut() {
                *value += QM31::one();
            }
        }
    }

    // Capture before `proof` is shadowed below; the `Proof<QM31>` has no `.aux` field.
    let first_query = proof.aux.unsorted_query_locations[0];

    // Create a context with values from the proof.
    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);
    let proof_vars = proof.guess(&mut context);
    let statement = SimpleStatement::new(&mut context);
    verify(&mut context, &proof_vars, &config, &statement);
    let context = context.finalize(true);

    let result = novalue_circuit.check(context.values());
    match proof_modifier {
        ProofModifier::None => {
            result.unwrap();
        }
        ProofModifier::WrongTraceAuthPath => {
            let err = result.unwrap_err();
            // The error should be when comparing the main trace root (word-by-word).
            let expected_value = context.get(*proof_vars.trace_root.0[0].get());
            assert!(err.contains(&expected_value.to_string()));
        }
        ProofModifier::WrongFriAuthPath => {
            let err = result.unwrap_err();
            // The error should be when comparing the first layer Merkle root (word-by-word).
            let expected_value =
                context.get(*proof_vars.fri.commit.layer_commitments[0].0[0].get());
            assert!(err.contains(&expected_value.to_string()));
        }
        ProofModifier::WrongFriWitness => {
            let err = result.unwrap_err();
            // The error should be when validating the query position inside its witness
            // coset. `validate_query_position_in_coset` selects the value at the query's
            // local index (`queried_pos`'s low `last_step` bits) in the last-layer coset.
            let fold_steps = compute_all_fold_steps(
                config.fri.log_trace_size - config.fri.log_n_last_layer_coefs,
                config.fri.fold_step,
            );
            let last_step = *fold_steps.last().unwrap();
            let shift: usize = fold_steps.iter().sum::<usize>() - last_step;
            let queried_pos = first_query >> shift;
            let index_in_coset = queried_pos & ((1 << last_step) - 1);
            let expected_value =
                context.get(proof_vars.fri.witness.0.last().unwrap()[0][index_in_coset]);
            assert!(err.contains(&expected_value.to_string()));
        }
    }

    // Make sure we got the same circuit.
    assert_eq!(context.circuit(), &novalue_circuit);

    novalue_circuit.check_yields();
    println!("Stats: {:?}", context.stats());
}

#[rstest]
#[case::fold_step_1(1)]
#[case::fold_step_2(2)]
#[case::fold_step_3(3)]
#[case::fold_step_5(5)]
fn test_proof_info(#[case] fold_step: u32) {
    let (_components, claim, pcs_config, proof, interaction_pow_nonce, channel_salt) =
        create_proof_with_fold_step(fold_step);

    let components = simple_statement_components::<NoValue>();
    let config = ProofConfig::new(&components, PREPROCESSED_COLUMN_LOG_SIZES.len(), &pcs_config, 8);
    let info = ProofInfo::from_config(&config);
    let circuit_proof =
        proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);
    let mut serialized = vec![];
    circuit_proof.serialize(&mut serialized);
    assert_eq!(serialized.len(), info.total_bytes());
    println!("fold_step={fold_step}:\n{info}");
}
