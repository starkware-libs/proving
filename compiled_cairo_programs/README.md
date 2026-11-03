# Compiled Cairo programs

The compiled Cairo0 programs the proving pipeline runs or commits to, built in the starkware
repo and copied here:

- `leaf_simple_bootloader_compiled.json`: the leaf simple bootloader, the program the `production`
  and `canonical_small` registries (`circuit_registry_definitions/`) build their leaf verifier
  circuits on.
- `aggregator_wrapper_compiled.json`: the aggregator wrapper, the leaf whose program hash those two
  registries' applicative unpacking verifier takes.
- `privacy_simple_bootloader_compiled.json`: the privacy simple bootloader, the program the
  `privacy_large_proofs` registry builds its leaf verifier circuits on. `privacy_common` embeds it
  for `privacy_prove` and `privacy_circuit_verify`.

Where each program was built in starkware is commented next to its program hash snapshot, in
`crates/stwo_run_and_prove_recursive_tree/src/tests.rs` for the leaf simple bootloader and the
aggregator wrapper and in `crates/privacy_prove/src/tests.rs` for the privacy simple bootloader.
