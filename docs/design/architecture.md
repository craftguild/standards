# Architecture

Normative, language-neutral contracts and conformance data live in `spec/` and
`conformance/`. A versioned `craftguild.toml` identifies distribution artifacts,
source roles, adapters, policies and repository checks. Rust and npm are adapters,
not definitions of a product or distribution.

| Unit | Owns | May depend on |
| --- | --- | --- |
| craftguild-cli | Plain offline presentation | Standard library |
| craftguild-core | Build identity, input seals, CycloneDX validation | Serialization, hashes, schema validator |
| craftguild-cli-build | Application build observation and prepared input verification | core |
| craftguild-initializer | Installed init-only interface and own artifact identity | standards, core, cli, cli-build |
| craftguild-standards (engine) | Configuration, initialization, pinned context, build/distribution, adapters and policies | core, cli, cli-build |

The application supplies its command name, version, repository root and banner.
Never discover the root by walking into a parent repository. Git observation
requires a `.git` entry at that explicit root; linked-worktree files are allowed.
Build metadata is captured in the application's build script and has a small
structured representation usable by generator metadata and machine version paths.
Presentation never launches tools and never owns application argument grammar.

The [orchestration contract](orchestration.md) defines schema 2,
resolved execution context, bounded hooks, technical history, output publication
and the standalone entry protocol for ADR 0002. Published v0.1.0 remains unchanged.

## Distribution preparation

The normal build resolves one artifact, target, feature set and locked dependency
graph, then builds required assets, records actual bundled inputs, creates and
validates CycloneDX, and invokes the compiler with the prepared seal. Build-script
verification uses the same identity and seal definitions. Missing, changed or
unknown required inputs fail. A build script never invokes Cargo recursively.

Prepared seal schema 2 separates developer integrity evidence from the
consumer SBOM. Source inventories and hashes remain in local seals and receipts;
the SBOM contains dependency/component evidence and artifact identity, without
the source ledger or its aggregate digest. Existing schema 1 artifacts retain
their original read/verify behavior.

Cargo's target-filtered, feature-selected graph provides normal runtime edges.
Build, development and procedural macro implementation dependencies are excluded
from shipped libraries. An explicit supplemental component records code/assets
which a package manager cannot discover. npm inclusion follows bundler evidence
mapped to the lockfile, rather than all development dependencies. Independent
frontend distributions have their own artifact and SBOM.

Preparation is a transient computation, not persistent business state: inputs
move through resolve, asset build, seal, validation and compile in one invocation.
Failure leaves no valid replacement seal; successful preparation is atomically
published. Process output records steps and failures. There is no durable domain
lifecycle, so a business transition-history table is inapplicable. SBOM component
arrays and file inventories are technical collections, not business line items;
the Header-Detail baseline does not require artificial business records here.

## Quality

Inventory precedes role classification and measurement. Unknown roles and absent
measurements fail closed. The engine owns stable rule IDs and reporting; adapters
own syntax and language tools; policies own selectable cross-project constraints;
repository commands own product invariants. Verified generated exceptions waive
only size, never compilation, formatting, hashes, ownership or reproduction.

Publication exports explicit public paths into a fresh history. Private archives,
local coordination state and working history are never prerequisites for a build.


ADR 0004 [initialization](initialization.md) replaces engine/skill archive delivery
with fixed references, a small shared entry and matching embedded context. The
standards engine owns acquisition, verification and rules; consumers own their
configuration and product behavior. Only init/explicit setup acquires an engine. ADR 0006 separates the installed
initializer from the full pinned engine; both share the initialization policy.
