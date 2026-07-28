# circuit-params

Computes the per-component sizes of the leaf-prover verifier circuit for a range of verified trace
sizes, using the CANONICAL preprocessed trace config. It reports two circuits:

- the **leaf verifier** circuit, which verifies one Cairo proof (reported for every trace size), and
- the **multiverifier** circuit, which verifies two proofs of the leaf verifier circuit (reported
  once, for the largest trace size).

Entry point: `crates/circuit_params/src/main.rs`

## Build & run

    cargo run -p circuit-params -- --help

## Usage

Required flags:

- `--min-trace-log-size <N>`: smallest verified trace log size to measure (inclusive). A canonical
  Cairo trace commits its preprocessed sequence columns at `MAX_SEQUENCE_LOG_SIZE = 25`, so a real
  canonical leaf proof has `log_trace_size >= 25`.
- `--max-trace-log-size <N>`: largest verified trace log size to measure (inclusive).

Optional:

- `--log-blowup-factor <N>`: log blowup factor (1, 2, or 3, default 1) of both the verified Cairo
  proof and the circuit proofs.
- `--registry`: output a JSON circuit registry (see below). If omitted, prints the human-readable
  report instead.
- `--output-path <PATH>`: file to write the output to. Prints to stdout if omitted. Use this with
  `--registry`: the binary's tracing output also goes to stdout, so JSON printed there is
  interleaved with log lines and cannot be parsed.

Example:

    cargo run -p circuit-params -- \
      --min-trace-log-size 25 \
      --max-trace-log-size 25 \
      --registry \
      --output-path /abs/path/to/params.json

## Output formats

### Default (human-readable)

One line per circuit and trace size, giving each AIR component's padded log size and its usage
percentage (how much of the padded power-of-two component is actually used).

Can be used to choose circuit configurations and to find components whose size can be reduced.

### `--registry` (JSON)

A JSON circuit registry, with three top-level fields:

- `circuit_proof_configs`: a map from a config id (a string) to a config — its `log_blowup_factor`
  and the padded `component_log_sizes` circuits are padded to. Circuits proven using the same
  config can be verified using the same verifier circuit.
- `leaf_verifiers`: the leaf verifier circuits, each referencing its `config` (by id), its
  `trace_log_size`, the verified proof's `log_blowup_factor`, and its `circuit_hash`.
- `multiverifiers`: the multiverifier circuits, each referencing its own `config` (by id), the
  `input_configs` (by id) of the circuits whose proofs it verifies, and its `circuit_hash`.
