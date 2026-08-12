---
paths:
  - "crates/prover/**"
  - "crates/adapter/**"
  - "crates/cairo-air/**"
  - "crates/common/**"
  - "crates/cairo-serialize/**"
  - "crates/cairo-serialize-derive/**"
  - "crates/utils/**"
  - "crates/dev_utils/**"
  - "stwo_cairo_verifier/**"
---

# Guide: stwo-cairo (Cairo CPU AIR, Prover & Verifier)

The Cairo-specific AIR plus a full prover and verifier in both Rust and Cairo. The core
`stwo` library provides the generic proving/verification logic; this area adds the Cairo CPU
AIR and trace handling. The **Cairo** verifier enables recursive proof verification on-chain
(a Cairo program verifying a STARK proof of another Cairo execution).

Note: "Cairo" is overloaded — it is both a CPU architecture and a programming language
(sometimes "Cairo1"). This area contains a Cairo1 verifier for recursive proving.

References: [Cairo paper](https://eprint.iacr.org/2021/1063),
[Circle STARKs paper](https://eprint.iacr.org/2024/278). For math, see the `stwo-core-guide`
rule and load the math skills in `_index.md`.

## Layout

Rust prover side:

```
crates/
  prover/                      Core STARK prover for Cairo CPU traces  (pkg: stwo-cairo-prover)
    src/witness/components/    [AIR-GENERATED] 50+ opcode/builtin witness generators
  adapter/                     Converts Cairo VM traces → Stwo prover format
    src/opcodes.rs             Opcode state transitions
    src/builtins.rs            Builtin segment handling
    src/memory.rs              Memory trace management
  cairo-air/                   [SOUNDNESS-CRITICAL] Generated AIR-specific logic for the Cairo CPU
    src/components/            Component constraint definitions
    src/relations.rs           Relation IDs (generated hashes)
  common/                      Shared types, preprocessed columns  (pkg: stwo-cairo-common)
  cairo-serialize/             Proof serialization/deserialization
  cairo-serialize-derive/      Derive macros for serialization
  utils/                       Utility functions  (pkg: stwo-cairo-utils)
  dev_utils/                   Dev tools: prove, verify, run_and_prove, etc.
```

Cairo verifier side (repo root, a separate Cairo/Scarb workspace — recursive proving):

```
stwo_cairo_verifier/
  crates/
    cairo_verifier/            Main verifier entry point  (pkg: stwo_cairo_verifier)
    cairo_air/                 [SOUNDNESS-CRITICAL] Generated AIR-specific logic in Cairo  (pkg: stwo_cairo_air)
    circuit_verifier/          [SOUNDNESS-CRITICAL] Cairo verifier for circuit-based STARK proofs  (pkg: stwo_circuit_verifier)
    circuit_air/               [SOUNDNESS-CRITICAL] Generated circuit-AIR logic in Cairo  (pkg: stwo_circuit_air)
    verifier_core/             [SOUNDNESS-CRITICAL] Core verification logic
      src/fields/              [SOUNDNESS-CRITICAL] M31, CM31, QM31 field arithmetic
      src/vcs/                 [SECURITY-CRITICAL]  Vector commitment schemes
      src/channel/             [SECURITY-CRITICAL]  Fiat-Shamir channels
      src/pcs.cairo            [SOUNDNESS-CRITICAL] PCS verification
    constraint_framework/      Constraint framework in Cairo
    bounded_int/               Bounded integer operations
    verifier_utils/            Verifier utility functions
    cairo_verifier_mock/       Mock verifier for testing
```

## Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Prover language | Rust nightly | See `rust-toolchain.toml` |
| Cairo verifier language | Cairo (2024_07 edition) | Scarb, for recursive verification |
| Cairo VM | cairo-vm | Trace extraction from Cairo CPU execution |
| Test runner (Rust) | nextest | Custom profiles in `.config/nextest.toml` |
| Test runner (Cairo) | scarb test | Feature-matrix tested in CI |
| Hash functions | Blake2s, Blake2sM31, Poseidon252 | Configurable via verifier features |

## Commands

Prover (Rust), from the repo root / prover crates:

```bash
cargo build --release
# Tests need: RUST_MIN_STACK=4194304  RUSTFLAGS="-C target-cpu=native"
cargo nextest run --cargo-profile ci --features=slow-tests -j 1
# Wasm (prover, wasm64)
RUSTFLAGS='--cfg getrandom_backend="custom"' cargo check \
  --target wasm64-unknown-unknown -Z build-std=std,panic_abort \
  --package stwo-cairo-prover --release
# Wasm (cairo-air, no_std, wasm32)
cargo check --package cairo-air --no-default-features \
  --target wasm32-unknown-unknown --release
```

Cairo verifier, from `stwo_cairo_verifier/`:

```bash
scarb fmt --check
scarb lint --features=<feature> --deny-warnings
scarb test --features=<feature> --package <package>
# Feature flags: poseidon252_verifier, qm31_opcode, blake_outputs_packing,
#                poseidon_outputs_packing
scarb --profile proving execute --package stwo_cairo_verifier \
  --features <feature> --print-resource-usage --output none --arguments-file <proof_file>
```

## Key Architectural Decisions

1. **Dual-language architecture** — Rust prover/verifier + Cairo verifier; the Cairo verifier
   enables recursive proving.
2. **Feature-gated hash functions** — verifier supports Blake2s/Poseidon252 via Scarb
   features; CI tests a matrix.
3. **AIR code generation** — component witness generators and constraint definitions are
   generated by **stwo-air-infra**. Files marked `// This file was created by the AIR team.`
   Manual edits are lost. See the `stwo-air-infra-guide` rule.
4. **Wasm compatibility** — prover (wasm64) and cairo-air (wasm32) must compile to Wasm;
   enforced by CI.
5. **Stable Rust compatibility** — all crates except `stwo-cairo-prover` and
   `stwo-cairo-dev-utils` must compile on stable.
6. **Optimization profiles** — `ci` profile optimizes all crates for testing, but keeps debug asserts
    and has `lto = false`; `adapter-release` optimizes the adapter.
