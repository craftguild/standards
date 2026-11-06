---
name: craftguild-standardize
description: Apply a pinned Craftguild standards version to a requested repository scope, including CLI identity, artifact SBOMs, quality policies and development/publication preparation. Product adoption does not authorize publication or service operations.
---

# Craftguild Standardize

Use the normative baseline recorded in `craftguild.toml`, currently 0.1.0.
Package and engine versions are recorded separately in the bundle build identity. Read the packaged [workflow reference](references/workflow.md) and only the
normative contracts relevant to the request. Run the bundled standards executable;
an installed skill bundle includes its corresponding rules and tool, so a sibling
standards checkout is unnecessary.

Compatible bundles additionally carry `assets/execution` and the
[execution contract](references/orchestration.md). This protocol
extension is not available from the published v0.1.0 revision. Use the
bundle's matching engine and entry files together; never invent a public pin.

Inspect repository instructions, design authorities, dirty work, CLI/machine
contracts and existing required checks before changing files. Classify proposed
changes against the request. Keep unresolved design, ownership, private-data or
operational questions explicit; do independent authorized work while they resolve.

Use artifact configuration and shared policies rather than copying checker logic.
Keep product grammar and invariants in the product. During build adoption, move
product references to the shared output layout; a historical internal path alone
does not require compatibility. Keep common engine conformance tests in standards.
Add consumer tests only for a concrete product or integration risk left uncovered
by existing checks and exposed by this migration. Use the
[workflow](references/workflow.md) to assign verification ownership and justify
exceptions. Run existing required gates and artifact acceptance; do not turn
adoption into a second engine test suite. The bundled standards remain the
authority for exact thresholds, roles and exceptions.

Use the implementation-only code-size convention and README outline in the
standards. Apply design-baselines before relevant domain modeling; record history
and Header-Detail decisions before implementing them. Do not redesign unrelated
models merely to introduce these conventions.

Preserve product behavior, public interfaces, ABI, generated ownership/provenance,
running state, unrelated dirty work and original evaluation material. Reconcile
internal path references and contradictory instructions/checkers/design rules in
the same requested migration. Preserve required coverage when replacing obsolete
adapters and their tests; record its shared or product owner before removing a gate.

Standardization alone does not authorize history rewriting, publication,
installation, service start/stop, credential changes or wider migrations. Honor
the active request's authorization. Stop after stage-one standards publication;
stage-two product adoption requires a later request. Never treat a failed or
unexecuted gate as success. Report remaining scope and evidence precisely.


The ADR 0003 source extension supports optional schema 2 standard Rust output
layout, `[[installers]]`, `installer [--check]` and receipt-backed `package`.
Published v0.2.0 does not provide these additions. Use a matching engine/entry
source export for development; do not invent a public pin. Generation belongs to
the common engine, and both the shell template and generated scripts remain
measured implementation. Package requires clean release receipts at the current
full clean HEAD and an existing engine cache; it never builds or acquires tools.
Download installation uses verified release assets and differs from copy-only
source-build install. Keep network/sudo stubbed and HOME/destinations temporary
in tests; never use real user state to qualify installation.
