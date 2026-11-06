---
name: craftguild-standardize
description: Apply a fixed Craftguild standards engine to a requested repository scope, preserving product behavior and verification ownership.
---

# Craftguild Standardize

Read repository instructions, accepted designs, dirty work and required checks.
In initialized consumers run `make prime` (or `python3 standards/launcher.py prime`),
then the relevant `--topic NAME` documents. The fixed reference selects the engine
and embedded rules. Missing cache requires explicit `make setup`. Never substitute
another installed tool's rules, invent a public pin or copy rules/binaries into
the consumer. Published v0.3.0 does not implement init/prime or flat reference
acquisition; source development and publication are separate qualifications.

In the standards source checkout use the canonical [contracts](../../spec/README.md),
[configuration](../../docs/design/configuration.md), [execution design](../../docs/design/orchestration.md)
and [workflow](references/workflow.md). Its bootstrap is a development tool and
cannot impersonate a clean published engine.

Use actual artifact/source/adapter/policy configuration. Keep product grammar,
SDK/ABI, generators and domain invariants with their product. Standards owns
common engine, output, seal, receipt, font renderer, measurement, acquisition,
installer, package and release tests. Retire removed local-adapter tests after
identifying their upstream owner; do not move them into a product runtime crate
or retain figlet-rs/vendor solely to retest standards. Add product acceptance only
for a concrete uncovered integration risk; preserve required existing coverage.
Report general counter gaps upstream instead of copying counters into products.

New Rust adoption uses standard outputs. Keep the normative baseline, engine
version, full source revision and compatibility identifiers distinct. Common-crate
Git declarations use that same full revision; init does not edit Cargo/npm inputs.
Run required checks and artifact acceptance. Apply implementation-only code-size,
the README outline and applicable design-baselines before domain modeling.

Preserve unrelated changes, behavior, ABI, generated provenance, user data and
running state. Reconcile contradictory current instructions/checkers in the same
requested migration. Initialization is unfinished adoption and no conformance
certificate. Publication, product migration, installation and service operations
require the active request's authorization. Stage-one standards work does not
itself authorize stage-two adoption. Report executed evidence and unexecuted hosts
precisely; never claim an unexecuted or failed check passed.


The ADR 0006 source initializer `craftguild-standards` exposes only `init [--force]`
and offline presentation. Repository operations use the pinned full
`craftguild-standards-engine` through Make/launcher. Init selects its invoking
published version/revision: upgrades are allowed; downgrades and same-version
revision replacement need force. Force does not bypass verification or ownership.
Exact retries reuse the reference; `make setup` only restores that fixed engine.
Manually align common-crate Git revisions with the reported full revision and
complete product adaptation/checks. Published v0.4.0 retains its old single-CLI
behavior; do not assume it accepts the new flag or invent a public source pin.
