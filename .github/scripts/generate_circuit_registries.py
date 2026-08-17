#!/usr/bin/env python3
"""
Generates each supported-circuits registry (the `circuit_params --registry` JSON), at
`<output dir>/<name>/registry.json`.
"""

import argparse
import json
import subprocess
from pathlib import Path

# Each registry's `circuit_registry_definitions/<name>/definition.json` names the params its
# proofs are produced with (recorded in the registry, so they are also what the proving binaries
# then run with), the program its leaves attest to, and the (inclusive) range of verified Cairo
# trace log sizes it covers:
# - `min_trace_log_size` is bounded below by the preprocessed-trace variant's sequence-column log
#   height (20 for canonical_small, 25 for canonical).
# - Changing the range, either params file or the program changes every circuit hash (all of a
#   registry's circuits are padded to the elementwise max over the range), so it requires
#   regenerating the recursive tree's goldens and the committed registry. Widening also costs
#   proving every leaf at the largest member's shape.
#
# `production` is what the backend proves with (its definition is also the source of the Cairo
# circuit verifier's generated consts — see `circuit_params/tests/cairo_consts_test.rs`);
# `canonical_small` is the cheap one the tests and goldens use, so its inputs live next to them.
REGISTRY_NAMES = ["production", "canonical_small"]


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

    for name in REGISTRY_NAMES:
        definition = json.loads(
            Path(f"circuit_registry_definitions/{name}/definition.json").read_text()
        )
        registry_dir = Path(args.output_dir) / name
        registry_dir.mkdir(parents=True, exist_ok=True)
        command = [
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
        ]
        pad_to = definition.get("pad_to_component_log_sizes")
        if pad_to is not None:
            command += ["--pad-to-component-log-sizes", json.dumps(pad_to)]
        subprocess.run(command, check=True)


if __name__ == "__main__":
    main()
