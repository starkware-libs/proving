# Stwo Cairo Verifier Workspace

[Cairo](https://github.com/starkware-libs/cairo) code for verifying
[Stwo](../crates/stwo) proofs. The main deliverable is `stwo_circuit_verifier`, a Cairo
program that verifies circuit-based STARK proofs, built on the shared verifier libraries
(`verifier_core`, `verifier_utils`, `constraint_framework`, `bounded_int`).

> **Note:** The Cairo CPU-AIR verifiers (`cairo_verifier`, `cairo_air`,
> `cairo_verifier_mock`) are frozen and no longer developed on main. They are kept on the
> `cairo-verifier-frozen` branch (base tagged `cairo-verifier-frozen-base`), cut from the
> last main commit containing them, from which their artifacts are built and uploaded.

## Install dependencies

[Install asdf](https://asdf-vm.com/guide/getting-started.html#_3-install-asdf) and run:

```bash
asdf plugin add scarb
asdf plugin add starknet-foundry
asdf install
```

## Run tests

Make sure [dependencies are installed](#install-dependencies). Run:

```bash
scarb test
```

## Profile tests

Modify [`Scarb.toml`](./Scarb.toml) to use [Starknet Foundry](https://github.com/foundry-rs/starknet-foundry).

```diff
[dev-dependencies]
- cairo_test = "2.11.4"
+ snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry", tag = "v0.33.0" }
+ assert_macros = "2.9.2"
+
+ [scripts]
+ test = "snforge test --max-n-steps 100000000"
```

Generate trace for all tests.

<!-- TODO(andrew): Debug error on Linux. -->
> :warning: Command produces errors on Linux.

```bash
scarb test -- --save-trace-data
```

Install [cairo-profiler](https://github.com/software-mansion/cairo-profiler) and run:

```bash
# Replace `TEST_NAME` with the name of the test you want profiled.
cairo-profiler ./snfoundry_trace/TEST_NAME.json
```

Visualise profile in the browser.

```bash
# Once opened navigate to `Sample -> steps`.
go tool pprof -http=":8000" profile.pb.gz
```

## Run the circuit verifier

To execute the circuit verifier on a proof and estimate its resource usage (total number
of steps, builtin usage):

```sh
scarb --profile proving execute \
    --package stwo_circuit_verifier \
    --features qm31_opcode \
    --arguments-file crates/circuit_air/test_data/proof.json \
    --print-resource-usage \
    --output none
```
