# ADR 0004: Repository initialization and prime

Status: Accepted; implemented in source

[ADR 0006](0006-initializer-cli-and-controlled-engine-upgrades.md) supersedes the
single public CLI, existing-pin retention and cross-version init delegation
decisions below. ADR 0006 is implemented in source; the historical published
v0.4.0 binary and its behavior described here remain unchanged.

This replaces the execution/skill bundle distribution decision in
[ADR 0002](0002-standards-owned-build-orchestration.md) and the standards-specific
bundle additions in [ADR 0003](0003-standard-rust-outputs-and-release-installers.md).
Other build, output, receipt, SBOM and installation contracts remain in force.
Implement [ADR 0005](0005-explicit-major-release-control.md) first so that this
breaking distribution change cannot automatically promote a 0.x group to 1.0.0.
The published v0.2.0/v0.3.0 engines do not provide these commands or this delivery
model. Acceptance alone is not host qualification; source implementation and executed
boundaries are recorded in the [verification guide](../guides/verification.md#adr-0004-source-qualification-2026-11-06).

## Context

Current adoption copies execution archives, rules, documentation and a full skill
into each consumer. Products also supply host selection and archive verification
code. These copies make repositories larger and give agents several infrastructure
components to maintain during an otherwise small integration.

The renderer review found an additional ownership problem: a font-reader test
from a retired local standards adapter was retained in the product, together with
a figlet-rs development dependency and vendored patch. Its actual consumer CLI
does not need that dependency. Keeping every old test after transferring its
responsibility defeats build consolidation.

CraftGuild needs a small repository entry and a way for an agent to read the
matching rules on demand. Initialization prepares that workflow; it does not
complete the product migration or certify conformance.

## Decision

### Initialize a repository, then let the agent finish adoption

Add these interfaces to the existing public command, without renaming it:

    craftguild-standards init
    craftguild-standards prime
    craftguild-standards prime --topic NAME

The workflow is initialization, prime, product-specific configuration and code
changes, then the required checks and artifact acceptance. The current working
directory is the explicit source root; do not discover a different root by walking
into a parent repository or run git init on the user's behalf.

`init` provides:

- A versioned fixed reference for the engine, rules and generated entry.
- A thin shared Make/launcher entry, including the existing explicit setup entry.
- A short repository-local craftguild-standardize skill and a common AGENTS block
  that instruct agents to read prime before adoption work.
- A craftguild.toml starter when that file is absent.
- An ignore rule for the repository-local engine cache.

The starter is valid TOML with the supported schema/baseline and explicit unfinished
configuration. Use an empty artifacts array and commented examples for artifact,
adapter and policy declarations, rather than inventing product packages, commands,
source roles or dependency boundaries. Explain that it is not a runnable build
configuration yet. New Rust adoption uses the standard output layout. Keep the
normal build/check configuration validator strict; an empty artifact selection
must still fail there.

Existing craftguild.toml and product-specific instructions are preserved. Create a
top-level Makefile only when absent; otherwise provide the shared include and
explain the remaining wiring without replacing existing recipes. AGENTS updates
are restricted to the known common block; preserve repository contracts. Do not
alter Cargo/npm dependencies, domain code, generated product outputs or global
agent settings as part of initialization.

Identical generated files are a no-op on reexecution. Reuse an existing fixed
reference without silently changing its version. For conflicting generated files
or a reference requiring migration, report the conflict and the needed explicit
adaptation rather than overwrite it or fall back to the old bundle model.

### Acquire a verified engine through the shared entry

New initialization fixes the clean published version/revision of the invoking
tool and its matching supported host asset. An unpublished source build cannot
invent a public revision or relabel older assets as implementing this ADR.
Tests use isolated release fixtures and transport stubs.

The committed reference identifies the engine version, complete source revision,
normative baseline, compatibility information and exact HTTPS asset digests.
The generated entry also has recorded ownership and hashes. Keep one reference
authority for engine/rules provenance and use the same full revision in consumer
Cargo declarations when completing adoption. Initializing does not itself edit
those declarations.

Standards supplies the acquisition and verification implementation. Products call
it instead of writing their own downloader or host selector. Initial `init` and
explicit `make setup` may acquire the pinned engine. Ordinary build, prepare,
test/check, release, install, package and prime never acquire an engine. A missing
or invalid cache fails with setup instructions, without downloading a tool,
choosing Latest, compiling a replacement or consulting a sibling checkout.

Use `.craftguild` inside the explicit repository root as the default cache, with
entries distinguished by their fixed identity and host. `init` adds
`/.craftguild/` to the root .gitignore without duplicate rules or loss of existing
content. It does not untrack already tracked files. Cache data is not a committed
build input or a user-global installation.

Download to owned staging, verify the expected bytes and build identity, then
publish the cache entry. Before operational execution, the shared entry verifies
the fixed revision, hashes and compatibility. Tampering, mismatches, unsupported
hosts and unavailable assets fail; no unverified binary is executed. Reuse valid
entries without mutable release lookup and preserve the previous valid entry on
failed replacement.

These guarantees apply to the supplied common entry. A consumer controls its own
repository and can bypass that entry or use another engine; standards cannot
force its acquisition path or claim to validate such execution.

### Read matching rules with prime

Embed the canonical rules and relevant adoption/configuration guidance into the
standards engine. The embedded texts are owned build inputs and correspond to
that engine's source revision. Do not require a copied spec/docs tree, a separately
downloaded rules archive or a network request to display them.

Repository prime uses the verified cached engine selected by the fixed reference,
including when the invoking installed tool has another version. Missing setup or
an incompatible reference fails clearly rather than presenting another version's
rules as the repository's authority.

The default output is concise Markdown containing:

- The normative baseline, engine version and full source revision.
- The repository configuration's preparation status and the next adoption steps.
- The shared build/output and verification-ownership rules relevant to adoption.
- Available topic names and commands for reading their full matching documents.

`--topic NAME` emits the corresponding embedded document; unknown topics fail with
the available choices. Output goes to stdout without a banner or terminal escapes.
Prime has no network, file-write, generator, compiler or check side effects.

Prime must work after init while the starter is incomplete. Configuration absence,
syntax errors or unfinished declarations are reported as preparation status, not
conformance success. Context retrieval is independent of successful operational
configuration validation. Do not weaken build/check gates to make prime work.
References in the output must be usable through topics or pinned source links,
without assuming a copied Markdown tree exists in the consumer checkout.

### Keep verification with its owner

The short local skill directs agents to prime and then the relevant topics. Prime
explains the division of responsibility before asking agents to adapt a product:

- Standards owns common context/output resolution, process cleanup, seals,
  receipts, generic acquisition/install/package/release behavior, measurement
  adapters and the font renderer used by its own tooling.
- Products own actual runtime requirements, SDK and ABI constraints, canonical
  generators, payload composition, domain behavior and their integration hooks.

Retire tests of a removed local common adapter after identifying their upstream
owner. Do not preserve them by moving them into a product runtime crate. A product
must not acquire figlet-rs or a vendor tree solely to retest standards internals.
Extend product acceptance only for a concrete uncovered integration risk.

General measurement gaps, including byte-identical Ruby/Groovy extension aliases,
belong upstream. Record unsupported capabilities instead of copying counter logic
into every repository. Implementing those extra aliases is separate from this
initialization/context feature. Preserve required SDK, ABI, domain and generation
checks; transferring common responsibility does not waive them.

### Retire bundle distribution

Future standards delivery consists of host binaries, matching SBOMs, checksums and
versioned release metadata. The metadata identifies the exact full revision,
supported host, engine compatibility and asset hashes without execution/skill
archive fields. Rules travel inside the binary; init creates the small local
entry and skill. Consumers commit their configuration, fixed reference and thin
entry, not standards archives, binaries or complete rules/documentation trees.

Remove execution/skill archive producers, old bundle loaders/extractors,
archive-dependent CI and their obsolete conformance tests from the new workflow.
Replace their still-required integrity coverage with tests of the new owner.
Do not add a compatibility loader, an old archive option or a legacy fallback.
Keep unrelated product package archives and the standalone download installer
from ADR 0003; they are different distribution responsibilities.

Published tags, assets, notes and historical receipts remain unchanged. Existing
consumers require an explicit migration and a release implementing this ADR;
initialization does not migrate external product repositories automatically.

### Bound the technical operation

Check ownership, path safety and conflicts before publishing generated files.
Stage replacements and downloads under owned temporary paths. A failed init must
not report successful initialization or leave a mixed set of new managed files;
restore only its own edits and preserve unrelated content and valid cached engines.
Recognize identical successful work on retry without duplicating instructions or
ignore rules. Refuse symlink/path escapes and unexpected concurrent changes.

This is a transient technical lifecycle, not a business entity. Staging results,
the immutable fixed reference and cache verification metadata retain technical
provenance. There is no business state/history table or Header-Detail aggregate;
file and asset inventories are technical collections.

## Alternatives and consequences

Keeping the old bundles would retain archive handling and large copied assets in
every consumer. Supporting both modes would prolong the same duplication. The
owner accepts the breaking replacement rather than paying that compatibility cost.

A user-global shared cache could avoid repeated downloads but adds shared mutable
state between repositories. Repository-local caching retains independent pins and
owned setup. It is ignored by Git, so it does not enlarge the committed source.
An installed global tool is an initialization convenience, not a mutable authority
for an initialized repository's ordinary builds.

Initialization cannot infer all product contracts. An explicit unfinished starter
and read-only context command make that boundary visible without pretending to
complete adoption. Build prerequisites and locked language dependencies remain a
separate preparation responsibility from engine acquisition.

## Acceptance and rollout

- Cover fresh init, incomplete configuration followed by prime, existing product
  configuration/instructions, identical retries, conflicts, interrupted writes,
  missing/duplicate ignore rules and owned rollback.
- Use isolated roots and stubbed transport to cover both published host targets,
  missing assets, unsupported hosts, incomplete downloads, wrong revision/hash,
  incompatible engines, cache reuse and rejection after tampering.
- Prove that no ordinary operation acquires an engine and that prime works offline
  without modifying files or invoking product code, including with incomplete or
  malformed configuration. Verify matching-topic output and version identity.
- Demonstrate one standalone consumer workflow using only a pin, thin entry and
  configuration, with no legacy archives, sibling repository, copied rules or
  runtime font-renderer dependency introduced by adoption.
- Keep common failure-matrix tests in standards. Reuse product-specific acceptance
  for actual artifacts, outputs, offline presentation and exact embedded SBOMs.
- Update canonical specs, packaging/CI, docs and the source skill together during
  implementation. Preserve Rust 1.85 consumer compatibility, standard output
  locations and local-only source integrity ledgers. Run required checks and the
  implementation-only code-size assessment; report unexecuted hosts separately.

Implementation belongs in standards after ADR 0005. Product migrations, real-user
installation, commits, release preparation, push and publication are outside this
goal. The implementation prompt is registered in bd as `standards-g6b`, depending
on `standards-a0c` for ADR 0005. Registration does not start either implementation.
