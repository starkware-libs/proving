# Skill & Rule Registry — Proving Monorepo

Agent context here comes in two forms:

- **Domain guides** — orientation for each area of the monorepo (layout, commands,
  architectural decisions). These are **path-scoped rules** in `.claude/rules/`, not skills:
  each one auto-loads when you read a file in its area's crates, so there is nothing to load
  by hand. See the Repository Map in the root `CLAUDE.md`.
- **Math & review skills** — cross-cutting cryptographic foundations and review checklists,
  loaded on demand from `.claude/skills/`. These apply to the **whole** proving stack (stwo,
  stwo-cairo, stwo-circuits, proving-utils, stwo-air-infra), not just the core library —
  every area proves or verifies over the same Circle STARK / M31 machinery.

## Domain Guides (path-scoped rules in `.claude/rules/`)

Auto-loaded by the `paths` frontmatter in each rule when you touch the matching crates — no
manual loading required.

| Guide | Rule file | Auto-loads when working on |
|-------|-----------|----------------------------|
| stwo (core) | `.claude/rules/stwo-core-guide.md` | The core prover/verifier library |
| stwo-cairo | `.claude/rules/stwo-cairo-guide.md` | The Cairo CPU AIR, Rust prover, or Cairo verifier |
| stwo-circuits | `.claude/rules/stwo-circuits-guide.md` | The circuit DSL or circuit-based prover/verifier |
| proving-utils | `.claude/rules/proving-utils-guide.md` | Run-and-prove, recursive trees, or privacy CLIs |
| stwo-air-infra | `.claude/rules/stwo-air-infra-guide.md` | AIR code generation, or regenerating AIR-generated code |

## Math & Review — Tier 1 (Mathematical Foundations + Review)

| Skill | File | Load When |
|-------|------|-----------|
| ZK-STARK Foundations | `zk-stark-foundations/SKILL.md` | Working on any proof system code |
| Circle STARK Mathematics | `circle-stark-mathematics/SKILL.md` | Modifying circle points, cosets, domains, FFT, polynomials |
| AIR Constraint Engineering | `air-constraint-engineering/SKILL.md` | Defining/reviewing constraints, logup, EvalAtRow |
| Finite Field Arithmetic | `finite-field-arithmetic/SKILL.md` | Modifying M31/CM31/QM31 ops, SIMD field code |
| Soundness Review Checklist | `soundness-review-checklist/SKILL.md` | Reviewing ANY soundness-critical change |
| Security Review Checklist | `security-review-checklist/SKILL.md` | Reviewing security-critical changes |

## Math & Review — Tier 2 (Protocol Specifics)

| Skill | File | Load When |
|-------|------|-----------|
| FRI Protocol | `fri-protocol/SKILL.md` | Modifying FRI prover/verifier, parameters, folding |
| Performance Optimization | `performance-optimization/SKILL.md` | Benchmarking, SIMD, memory, profiling |
| Testing Strategy | `testing-strategy/SKILL.md` | Adding tests, reviewing coverage, debugging failures |

## Math & Review — Tier 3 (Operations)

| Skill | File | Load When |
|-------|------|-----------|
| Rust Codebase Conventions | `rust-codebase-conventions/SKILL.md` | Contributing code, understanding patterns |
| Debugging ZKP | `debugging-zkp/SKILL.md` | Proof failures, constraint debugging |

## Living Documents

| Document | File | Purpose |
|----------|------|---------|
| Divergence Log | `paper-implementation-divergence-log.md` | Paper vs code divergences (READ BEFORE MODIFYING THEORY CODE) |

## Distilled Theory References

| Document | File | Purpose |
|----------|------|---------|
| Distillation Index | `.agents/papers/llm/INDEX.llm.md` | Entry point and notation map for theory references |
| Circle STARK Distillation | `.agents/papers/llm/Circle_STARKs.llm.md` | Canonical Circle STARK definitions, algorithms, invariants |
| STWO Distillation | `.agents/papers/llm/Stwo_Whitepaper.llm.md` | Canonical STWO protocol model, soundness assumptions, parameters |

## Loading Protocol

1. The **domain guide** for the area you are working in auto-loads (path-scoped rule in
   `.claude/rules/`) — no action needed.
2. Load `.agents/papers/llm/INDEX.llm.md` to map concepts and anchors.
3. Load the relevant distilled paper file(s) from `.agents/papers/llm/`.
4. Always load `paper-implementation-divergence-log.md` before modifying any
   theoretically-grounded component.
5. Load the most specific relevant Tier 1 skill for the domain you're working in.
6. For reviews, load the appropriate checklist skill.
7. Tier 2 and 3 skills are loaded as needed for context.
