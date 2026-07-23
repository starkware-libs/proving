---
paths:
  - "crates/air_infra/**"
  - "crates/airs/**"
  - "crates/air_code_gen/**"
  - "crates/air_compile/**"
  - "crates/air_common/**"
  - "crates/eval_air_fn_constraints/**"
  - "outputs/compiled_casm_air/**"
  - "outputs/compiled_circuit_air/**"
  - "test_data/code_gen_regression/**"
---

# Guide: stwo-air-infra (AIR Code-Generation Toolchain)

The toolchain that **generates** the AIR code consumed by stwo-cairo and stwo-circuits: it
compiles AIR definitions into constraint evaluators and witness generators emitted as Rust
and Cairo. Files it emits carry `// This file was created by the AIR team.` and must never be
hand-edited in the consuming areas — change them here and regenerate.

This area is the source of truth for everything marked `[AIR-GENERATED]` elsewhere. Because a
codegen bug propagates into `[SOUNDNESS-CRITICAL]` constraint code, treat changes to the
generator with the same care: load `air-constraint-engineering` and the soundness review
checklist from `_index.md`.

## Layout

```
crates/
  air_infra/               Top-level AIR infrastructure entry point
    src/airs/              AIR definitions wiring
    src/casm_state.rs      CASM state modeling
    src/felt252_id_memory/ Felt252 / id-memory modeling
    src/range_check.rs     Range-check AIR pieces
  airs/                    AIR definitions (the inputs to codegen)
    src/casm/              CASM (Cairo CPU) AIR
    src/circuit/           Circuit AIR
    src/examples/          Example AIRs
    src/convolution_utils/, src/felt252_utils/
  air_code_gen/            Code generators — emit constraint/witness code
    src/rust/              Rust code emitter
    src/cairo/             Cairo code emitter
    src/circuit/           Circuit-target emitter
    src/supported_components.rs
  air_compile/             Compiles AIR definitions into compiled structs
    src/compiled_structs.rs
  air_common/              Shared utilities across the AIR crates
  eval_air_fn_constraints/ Constraint-function evaluation
    src/logup.rs           LogUp evaluation
    src/scope.rs, src/assignment.rs
  outputs/compiled_casm_air/       Compiled CASM AIR artifacts
  outputs/compiled_circuit_air/    Compiled circuit AIR artifacts
  test_data/code_gen_regression/     Regression tests for generated code
```

## Workflow: regenerating AIR code

1. Change the AIR definition (`airs/`) or the emitter (`air_code_gen/`) here — never the
   generated files in stwo-cairo/stwo-circuits.
2. Regenerate and run `code_gen_regression` + the `test_generated_code_*` checks.
3. Because the output is `[SOUNDNESS-CRITICAL]` in the consuming areas, follow the Supervised
   Operation Boundary in the root `CLAUDE.md`: state the invariant, how it's preserved, and
   the verifying test; get approval.

## Notes

- `compiled_casm_air` / `compiled_circuit_air` hold compiled AIR artifacts; treat them as
  generated outputs of `air_compile`, not hand-maintained code.
- `code_gen_regression` is the guard that catches unintended changes in emitted code — keep
  it green and do not weaken it to make a diff pass.
