---
paths:
  - "crates/stwo/**"
  - "crates/constraint-framework/**"
  - "crates/air-utils/**"
  - "crates/air-utils-derive/**"
  - "crates/examples/**"
  - "crates/std-shims/**"
  - "ensure-verifier-no_std/**"
---

# Guide: stwo (Core Prover & Verifier)

The AIR-agnostic Circle STARK proving system — the cryptographic core every other area
builds on. Verifier code is `no_std`-compatible for on-chain deployment; the prover is
feature-gated behind `prover`.

For the mathematics behind any file below, load the math skills per `_index.md`
(circle-stark-mathematics, finite-field-arithmetic, fri-protocol, air-constraint-engineering).

## Layout

```
crates/
  stwo/                    Core prover + verifier library
    src/core/              [SOUNDNESS-CRITICAL] Verifier-side (no_std compatible)
      fields/              [SOUNDNESS-CRITICAL] M31, CM31, QM31 field arithmetic
      fri.rs               [SOUNDNESS-CRITICAL] FRI verifier
      verifier.rs          [SOUNDNESS-CRITICAL] Top-level STARK verifier
      pcs/                 [SOUNDNESS-CRITICAL] Polynomial commitment scheme
      channel/             [SECURITY-CRITICAL]  Fiat-Shamir channel
      circle.rs            [SOUNDNESS-CRITICAL] Circle group, cosets, domains
      constraints.rs       [SOUNDNESS-CRITICAL] Vanishing polynomials
      vcs/                 [SECURITY-CRITICAL]  Merkle tree (original)
      vcs_lifted/          [SECURITY-CRITICAL]  Lifted Merkle tree
      poly/                Circle polynomials, line polynomials, domains
      fft.rs               Butterfly operations
      proof.rs             Proof serialization
      proof_of_work.rs     [SECURITY-CRITICAL]  PoW verification
    src/prover/            [PERFORMANCE-CRITICAL] Prover-side (requires "prover" feature)
      backend/cpu/         CPU reference implementation
      backend/simd/        [PERFORMANCE-CRITICAL] SIMD-optimized backend (heavy unsafe)
        fft/               [PERFORMANCE-CRITICAL] SIMD FFT
        m31.rs             PackedM31 SIMD field ops
      fri.rs               FRI prover
      pcs/                 PCS prover + quotient ops
      lookups/             GKR + LogUp + sumcheck
      mempool.rs           Memory pool for allocation reuse
    benches/               Criterion benchmarks
  constraint-framework/    Framework for defining AIR constraints
    src/logup.rs           [SOUNDNESS-CRITICAL] LogUp interaction constraints
    src/prover/            Constraint evaluation (SIMD + CPU)
  air-utils/               Trace generation utilities
  air-utils-derive/        Proc macros for AIR utilities
  examples/                Example implementations (Blake, Poseidon, Fibonacci, etc.)
  std-shims/               no_std compatibility shims
ensure-verifier-no_std/    (repo root) CI gate: verifier compiles without std
```

## Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Field | Mersenne31 (M31) | CM31, QM31 extension tower. p = 2^31 - 1 |
| Proof system | Circle STARKs | FRI-based, circle group C(F_p) |
| Hash functions | Blake2s, Blake3, Poseidon252 | Blake2s primary for proofs |
| Benchmarks | Criterion | Benchmark suites + Poseidon example |


## Key Architectural Decisions

1. **Feature-gated prover** — the `prover` feature separates prover from verifier; the
   verifier is `no_std` for on-chain use.
2. **SIMD-first performance** — the SIMD backend is the primary path; the CPU backend is a
   reference. SIMD uses extensive `unsafe`.
3. **Lifted Merkle trees** — `vcs_lifted/` commits multiple polynomial sizes in one tree by
   lifting smaller polynomials to the largest domain.
4. **QM31 decomposition** — secure-field polynomials decompose into 4 base-field coordinate
   polynomials for commitment and FRI.
5. **Memory pooling** — `prover/mempool.rs` reuses allocations to avoid repeated large
   allocations during proving.

## Before modifying soundness-critical components

Load the corresponding math skill AND read the referenced paper section — do not proceed on
intuition. The `paper-implementation-divergence-log.md` skill is authoritative; any
undocumented divergence found during work MUST be added before proceeding.
