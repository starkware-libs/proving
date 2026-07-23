---
paths:
  - "crates/cairo-program-runner/**"
  - "crates/cairo-program-runner-lib/**"
  - "crates/vm_runner/**"
  - "crates/stwo_run_and_prove/**"
  - "crates/stwo_run_and_prove_common/**"
  - "crates/stwo_run_and_prove_recursive_tree/**"
  - "crates/privacy_prove/**"
  - "crates/privacy_circuit_verify/**"
  - "crates/circuit_params/**"
  - "crates/leaf_proof_format/**"
  - "crates/leaf_prover/**"
---

# Guide: proving-utils (Run-and-Prove, Recursive Trees & Privacy)

End-to-end orchestration on top of the proving stack: CLIs that run compiled Cairo programs,
generate stwo proofs, fold recursive proof trees, and drive privacy proving/verification.
This area is mostly **binaries and orchestration** — the cryptographic heavy lifting lives in
`stwo`, `stwo-cairo`, and `stwo-circuits`.

For math, see the relevant area guide rules (`stwo-cairo-guide`, `stwo-circuits-guide`) and
load the math skills in `_index.md`.

## Layout

```
crates/
  cairo-program-runner/         CLI: run a compiled Cairo program on the Cairo VM.
                                Optionally writes a Cairo PIE, or (proof mode) AIR
                                public/private inputs + encoded trace/memory.
                                Bin: `cairo_program_runner`.
  cairo-program-runner-lib/     Library backing the runner (Cairo VM + hint support).
  vm_runner/                    CLI: run a program in proof-mode config and adapt to
                                `stwo_cairo_adapter::ProverInput`; writes execution
                                resources and optionally prover input.  Bin: `stwo-vm-runner`.
  stwo_run_and_prove/           CLI: run a Cairo program and generate a STWO proof; optionally
                                verify and save output/debug data.  Bin: `stwo-run-and-prove`.
  stwo_run_and_prove_common/    Shared abstractions for the run_and_prove* crates — notably
                                the prover trait that lets consumers mock the slow prove step
                                in tests (`mock` feature → `MockProverTrait`).
  stwo_run_and_prove_recursive_tree/
                                Fold an entire applicative recursive proof tree above its
                                leaves into a single STWO root proof, in one binary invocation.
  privacy_prove/                Run Cairo programs and generate STWO proofs verifiable with a
                                cairo-circuit verifier.  (pkg: privacy-prove)
  privacy_circuit_verify/       Verify a proof with a cairo-circuit verifier.
                                (pkg: privacy-circuit-verify)
  leaf_prover/                  CLI: run Cairo programs, prove their runs, verify that proof
                                with a circuit-cairo verifier and output a proof of the run of
                                that circuit.  (pkg: leaf-prover)
  leaf_proof_format/            Describes the format of the output of the leaf prover.
                                (pkg: leaf-proof-format)
  circuit_params/               CLI: compute the leaf-prover verifier circuit's per-component
                                sizes for a range of trace sizes.  (pkg: circuit-params)
```


## Notes

Each binary has its own README under `crates/<crate>/README.md` for full usage.