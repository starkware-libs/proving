#!/usr/bin/env python3
"""
Decide which sub-repo CI flows to run, based on the changed files.
"""

import os
import subprocess
import sys
from dataclasses import dataclass, field
import tomllib
from typing import Optional


@dataclass(frozen=True)
class Crate:
    """
    Represents a Rust crate with additional information.
    """

    # Workspace-member path, matching an entry in Cargo.toml `workspace.members`.
    path: str
    # Cargo package name.
    package: str
    # Whether the crate should compile on stable toolchain.
    stable_check: bool = True


@dataclass(frozen=True)
class Group:
    """
    A set of crates, typically related logically.
    """

    crates: list[Crate]
    # Extra paths (not cargo crates) whose change should trigger this group's
    # CI.
    extra_paths: list[str] = field(default_factory=list)

    def trigger_paths(self) -> list[str]:
        """Paths whose modification triggers this group's CI flow."""
        return [f"{c.path}/" for c in self.crates] + list(self.extra_paths)


# The Rust crate groups, keyed by originating sub-repo. Source of truth for how
# workspace members map to CI flows.
#
# Skip format on this constant.
# fmt: off
RUST_GROUPS: dict[str, Group] = {
    "stwo": Group(
        crates=[
            Crate("crates/stwo", "stwo", stable_check=False),
            Crate("crates/air-utils", "stwo-air-utils", stable_check=False),
            Crate("crates/air-utils-derive", "stwo-air-utils-derive", stable_check=False),
            Crate("crates/constraint-framework", "stwo-constraint-framework", stable_check=False),
            Crate("crates/examples", "stwo-examples", stable_check=False),
            Crate("crates/std-shims", "std-shims", stable_check=False),
        ],
        extra_paths=[
            "ensure-verifier-no_std/",
            ".github/workflows/stwo-ci.yml",
        ],
    ),
    "stwo_cairo_prover": Group(
        crates=[
            Crate("crates/adapter", "stwo-cairo-adapter"),
            Crate("crates/cairo-serialize", "stwo-cairo-serialize"),
            Crate("crates/cairo-serialize-derive", "stwo-cairo-serialize-derive"),
            Crate("crates/utils", "stwo-cairo-utils"),
            Crate("crates/common", "stwo-cairo-common"),
            Crate("crates/cairo-air", "cairo-air"),
            Crate("crates/prover", "stwo-cairo-prover", stable_check=False),
            Crate("crates/dev_utils", "stwo-cairo-dev-utils", stable_check=False),
        ],
        extra_paths=[
            ".github/workflows/stwo-cairo-prover-ci.yml",
            # Changing this expected-proof fixture must re-run the slow-tests proof regression
            # that compares against it.
            "test_data/test_prove_verify_all_opcode_components/proof.json",
        ],
    ),
    "stwo_circuits": Group(
        crates=[
            Crate("crates/cairo_verifier", "circuit-cairo-verifier"),
            Crate("crates/circuits", "circuits"),
            Crate("crates/circuit_verifier", "circuit-verifier"),
            Crate("crates/circuit_cairo_serialize", "circuit-cairo-serialize"),
            Crate("crates/circuit_common", "circuit-common"),
            Crate("crates/circuit_multiverifier", "circuit-multiverifier"),
            Crate("crates/circuit_serialize", "circuit-serialize"),
            Crate("crates/circuit_prover", "circuit-prover", stable_check=False),
            Crate("crates/stark_verifier", "circuits-stark-verifier"),
            Crate("crates/stark_verifier_examples", "circuits-stark-verifier-examples", stable_check=False),
            Crate("crates/unpacker", "circuits-unpacker"),
        ],
        extra_paths=[".github/workflows/stwo-circuits-ci.yml"],
    ),
    "proving_utils": Group(
        crates=[
            Crate("crates/cairo-program-runner", "cairo-program-runner"),
            Crate("crates/cairo-program-runner-lib", "cairo-program-runner-lib"),
            Crate("crates/stwo_run_and_prove", "stwo-run-and-prove", stable_check=False),
            Crate("crates/stwo_run_and_prove_common", "stwo-run-and-prove-common", stable_check=False),
            Crate("crates/stwo_run_and_prove_recursive_tree", "stwo-run-and-prove-recursive-tree", stable_check=False),
            Crate("crates/vm_runner", "stwo-vm-runner"),
            Crate("crates/privacy_prove", "privacy-prove", stable_check=False),
            Crate("crates/privacy_circuit_verify", "privacy-circuit-verify"),
            Crate("crates/leaf_prover", "leaf-prover", stable_check=False),
            Crate("crates/leaf_proof_format", "leaf-proof-format"),
            Crate("crates/circuit_params", "circuit-params", stable_check=False),
            Crate("crates/circuit_registry", "circuit-registry"),
        ],
        extra_paths=[
            ".github/workflows/proving-utils-ci.yml",
            "circuit_registry_definitions/",
        ],
    ),
    "stwo_air_infra": Group(
        crates=[
            Crate("crates/air_code_gen", "air_code_gen"),
            Crate("crates/air_common", "air_common"),
            Crate("crates/air_compile", "air_compile"),
            Crate("crates/air_infra", "air_infra"),
            Crate("crates/airs", "airs"),
            Crate("crates/eval_air_fn_constraints", "eval_air_fn_constraints"),
        ],
        extra_paths=[
            ".github/workflows/stwo-air-infra-ci.yml",
            "visualizer/",
            "scripts/js_typecheck.sh",
            "scripts/visualizer_webapp_test.sh",
            "scripts/test_generated_code.py",
        ],
    ),
}
# fmt: on

# Paths that trigger the stwo-cairo-verifier CI (scarb), which covers the circuit Cairo
# verifier and the shared Cairo verifier libraries.
STWO_CAIRO_VERIFIER_GROUP: list[str] = [
    "stwo_cairo_verifier/",
    ".github/workflows/stwo-cairo-verifier-ci.yml",
    # The circuit verifier's execution fixture (see stwo-cairo-verifier-ci.yml).
    "crates/stwo_run_and_prove_recursive_tree/test_data/goldens/four_leaves/root.proof",
]

# Paths whose change forces every CI flow to run.
FULL_CI_GROUP: list[str] = [
    ".github/workflows/ci.yml",
    ".github/actions/",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rustfmt.toml",
    ".cargo/",
    "scripts/ci_dispatch.py",
]

# The flags written to GITHUB_OUTPUT that record whether to run or skip.
DECISION_OUTPUTS = [
    "run_stwo",
    "run_stwo_cairo_prover",
    "run_stwo_circuits",
    "run_proving_utils",
    "run_stwo_cairo_verifier",
    "run_stwo_air_infra",
]


def any_match(changed_paths: set[str], patterns: list[str]) -> bool:
    """
    Returns True if there exists a path in `changed_paths` that satisfies
    a pattern in `patterns`, else False.
    """

    def matches(path: str, pattern: str) -> bool:
        if pattern.endswith("/"):
            return path.startswith(pattern)
        return path == pattern

    return any(matches(path, pattern) for path in changed_paths for pattern in patterns)


def decide_flows_to_run(changed_paths: Optional[set[str]]) -> dict[str, bool]:
    """
    Given the changed paths, decide for each group whether to run/skip.
    """

    trigger_full_ci = changed_paths is None or any_match(
        changed_paths=changed_paths, patterns=FULL_CI_GROUP
    )
    if trigger_full_ci:
        return {name: True for name in DECISION_OUTPUTS}

    assert changed_paths is not None
    is_group_changed = {
        name: any_match(changed_paths=changed_paths, patterns=group.trigger_paths())
        for name, group in RUST_GROUPS.items()
    }

    run_stwo = is_group_changed["stwo"]
    # We don't trigger stwo-air-infra's CI on pull_request events if stwo or stwo-cairo changed,
    # even though it depends on stwo-cairo-common.
    run_stwo_air_infra = is_group_changed["stwo_air_infra"]
    run_stwo_cairo_prover = (
        run_stwo or run_stwo_air_infra or is_group_changed["stwo_cairo_prover"]
    )
    run_stwo_cairo_verifier = (
        any_match(changed_paths=changed_paths, patterns=STWO_CAIRO_VERIFIER_GROUP)
        or run_stwo_air_infra
    )
    run_stwo_circuits = run_stwo_cairo_prover or is_group_changed["stwo_circuits"]
    run_proving_utils = run_stwo_circuits or is_group_changed["proving_utils"]

    return {
        "run_stwo": run_stwo,
        "run_stwo_cairo_prover": run_stwo_cairo_prover,
        "run_stwo_circuits": run_stwo_circuits,
        "run_proving_utils": run_proving_utils,
        "run_stwo_cairo_verifier": run_stwo_cairo_verifier,
        "run_stwo_air_infra": run_stwo_air_infra,
    }


def changed_files(event_name: str) -> Optional[set[str]]:
    """
    Compute changed files for the current event, or None to run everything.
    """

    if event_name == "pull_request":
        base = os.environ.get("BASE_SHA")
        if not base:
            return None
        try:
            process = subprocess.run(
                ["git", "diff", "--name-only", f"{base}...HEAD"],
                capture_output=True,
                text=True,
                check=True,
            )
            return {line for line in process.stdout.splitlines() if line}
        except subprocess.CalledProcessError as err:
            # TODO(Leo): consider writing to GITHUB_STEP_SUMMARY so that this failure
            # is more visible.
            print(f"Error while running git diff: {err.stderr}")
    return None


def assert_crates_match_workspace():
    """
    Checks whether the set of paths of the workspace crates is equal to the set of paths of
    the crates hardcoded in `RUST_GROUPS`.
    """

    with open("Cargo.toml", "rb") as f:
        manifest = tomllib.load(f)

    manifest_crates = set(manifest["workspace"]["members"])
    crate_paths = set(
        crate.path for group in RUST_GROUPS.values() for crate in group.crates
    )

    # Check that `manifest_crates` and `crate_paths` are equal.
    missing_crates = manifest_crates.difference(crate_paths)
    stale_crates = crate_paths.difference(manifest_crates)
    if missing_crates or stale_crates:
        raise ValueError(
            f"Crates not dispatched: {missing_crates}\n"
            f"Crates that don't exist in the workspace: {stale_crates}"
        )


def main() -> int:
    assert_crates_match_workspace()
    event = os.environ.get("EVENT_NAME", "")
    changed = changed_files(event_name=event)
    decisions = decide_flows_to_run(changed_paths=changed)

    # Write to GITHUB_OUTPUT.
    gh_output = os.environ["GITHUB_OUTPUT"]
    with open(gh_output, "a") as f:
        # Write whether to run or skip individual workflows.
        for name in DECISION_OUTPUTS:
            f.write(f"{name}={str(decisions[name]).lower()}\n")

        # Write the packages belonging to each group. This step doesn't depend on
        # the decision to run or skip.
        for name, group in RUST_GROUPS.items():
            group_packages = " ".join(f"-p {crate.package}" for crate in group.crates)
            f.write(f"{name}_packages={group_packages}\n")

        # Write the packages to be checked with the stable toolchain. This step doesn't depend on
        # the decision to run or skip.
        stable_crates = " ".join(
            f"-p {crate.package}"
            for group in RUST_GROUPS.values()
            for crate in group.crates
            if crate.stable_check
        )
        f.write(f"stable_crates={stable_crates}\n")

    # Write the summary .md file shown in Github.
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as f:
            f.write("### CI dispatch\n\n")
            f.write(f"- event: `{event}`\n")
            n_changed_files = "ALL" if changed is None else str(len(changed))
            f.write(f"- changed files: {n_changed_files}\n\n")
            f.write("| output | value |\n| --- | --- |\n")
            for name in DECISION_OUTPUTS:
                f.write(f"| {name} | {str(decisions[name]).lower()} |\n")

    return 0


if __name__ == "__main__":
    sys.path.insert(0, os.path.dirname(__file__))
    sys.exit(main())
