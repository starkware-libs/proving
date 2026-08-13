#!/usr/bin/env python3
"""
Generates each supported-circuits registry (the `circuit_params --registry` JSON), at
`<output dir>/<name>/registry.json`.
"""

import argparse
import subprocess
from pathlib import Path

# Each registry definition names the files the proving binaries run with, and the (inclusive)
# range of verified Cairo trace log sizes its registry covers:
# - `min_trace_log_size` is bounded below by the preprocessed-trace variant's sequence-column log
#   height (20 for canonical_small, 25 for canonical).
# - Changing the range changes every circuit hash (all of a registry's circuits are padded to the
#   elementwise max over it), so it requires regenerating the recursive tree's goldens and the
#   committed registry. Widening also costs proving every leaf at the largest member's shape.
# TODO(yairv): add the production configs here.
REGISTRY_DEFINITIONS = [
    {
        "name": "canonical_small",
        "cairo_prover_params_json": (
            "crates/stwo_run_and_prove_recursive_tree/test_data/cairo_prover_params.json"
        ),
        "circuit_fri_config_json": (
            "crates/stwo_run_and_prove_recursive_tree/test_data/circuit_fri_config.json"
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
        help="Directory to write the registries under, one <name>/registry.json each.",
    )
    args = parser.parse_args()

    for definition in REGISTRY_DEFINITIONS:
        registry_dir = Path(args.output_dir) / definition["name"]
        registry_dir.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                args.circuit_params_binary,
                "--registry",
                "--cairo-prover-params-json",
                definition["cairo_prover_params_json"],
                "--circuit-fri-config-json",
                definition["circuit_fri_config_json"],
                "--program",
                definition["program"],
                "--min-trace-log-size",
                str(definition["min_trace_log_size"]),
                "--max-trace-log-size",
                str(definition["max_trace_log_size"]),
                "--output-path",
                str(registry_dir / "registry.json"),
            ],
            check=True,
        )


if __name__ == "__main__":
    main()
