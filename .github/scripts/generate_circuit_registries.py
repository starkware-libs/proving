#!/usr/bin/env python3
"""
Generates each circuit family's supported-circuits registry (the `circuit_params --registry`
JSON), at `<output dir>/<family>/registry.json`.
"""

import argparse
import subprocess
from pathlib import Path

# Each family names the files the proving binaries run with, and its (inclusive) range of verified
# Cairo trace log sizes:
# - `min_trace_log_size` is bounded below by the preprocessed-trace variant's sequence-column log
#   height (20 for canonical_small, 25 for canonical).
# - Changing the range changes every circuit hash (all of a family's circuits are padded to the
#   elementwise max over it). The binaries pad to the fixed `TARGET_PADDING_SIZES`, so a family
#   currently holds the single matching trace size.
#   TODO(yairv): widen the ranges once the binaries take their padding target from this registry.
# TODO(yairv): add the production configs here.
FAMILIES = [
    {
        "name": "canonical_small",
        "cairo_prover_params_json": (
            "crates/leaf_prover/tests/data/cairo_prover_params_canonical_small.json"
        ),
        "circuit_prover_params_json": (
            "crates/stwo_run_and_prove_recursive_tree/test_data/circuit_prover_params.json"
        ),
        "program": (
            "crates/stwo_run_and_prove_recursive_tree/test_data/"
            "leaf_simple_bootloader_compiled.json"
        ),
        "min_trace_log_size": 20,
        "max_trace_log_size": 20,
    },
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--circuit-params-binary",
        required=True,
        help="Path to the compiled circuit-params binary.",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write the registries under, one <family>/registry.json each.",
    )
    args = parser.parse_args()

    for family in FAMILIES:
        family_dir = Path(args.output_dir) / family["name"]
        family_dir.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                args.circuit_params_binary,
                "--registry",
                "--cairo-prover-params-json",
                family["cairo_prover_params_json"],
                "--circuit-prover-params-json",
                family["circuit_prover_params_json"],
                "--program",
                family["program"],
                "--min-trace-log-size",
                str(family["min_trace_log_size"]),
                "--max-trace-log-size",
                str(family["max_trace_log_size"]),
                "--output-path",
                str(family_dir / "registry.json"),
            ],
            check=True,
        )


if __name__ == "__main__":
    main()
