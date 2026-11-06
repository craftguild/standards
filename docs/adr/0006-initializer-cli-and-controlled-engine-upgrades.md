# ADR 0006: Initializer CLI and controlled engine upgrades

Status: Accepted; implemented in source

Date: 2026-11-07

This supersedes the single public CLI, existing-pin retention and cross-version
init delegation decisions in [ADR 0004](0004-repository-initialization-and-prime.md).
Its other initialization, acquisition, prime and verification-ownership decisions
remain in force. Published v0.4.0 still provides one full CLI and retains existing
pins; accepting this ADR does not change that binary or qualify a new release.

## Context

An initial `init` needs an installed tool, while ordinary repository operations
need a verified, repository-pinned engine. Giving both roles to the same public
command makes it harder to explain which version controls a build and how to
upgrade an initialized repository.

CraftGuild needs a small public interface for initialization and explicit engine
updates. The installed initializer selects its own published release, while the
repository entry continues to select the fixed engine for ordinary operations.
Accidental downgrades and same-version revision changes require an explicit
override.

## Decision

### Separate initialization from full engine operations

Distribute two executables in the same release group, with the same version and
full clean source revision:

| Command | Responsibility |
| --- | --- |
| `craftguild-standards` | `init [--force]` plus ordinary offline help, version and SBOM presentation |
| `craftguild-standards-engine` | Full engine operations, including initialization and prime |

The public download installer installs `craftguild-standards`. Running its `init`
acquires the matching engine for the repository. Give each executable its own
artifact identity, embedded SBOM, sidecar SBOM and checksums. Release metadata and
packaging distinguish the two executables; neither may impersonate the other.

The initializer does not expose build, check, release or prime as global operation
commands. Initialized repositories use their common Make/Python entry for those
operations. The engine and initializer share one initialization policy; direct
engine initialization must not provide a bypass around the update guards.

### Initialize or update to the invoking tool's release

The public interface is:

```sh
craftguild-standards init
craftguild-standards init --force
```

Use exactly cwd as the repository root. Select the invoking tool's own published
version and full revision, never Latest, an arbitrary requested version, an
unpublished source build or a sibling checkout. Release metadata and downloaded
engine identity must match that selection. The normative baseline remains
separate from the CLI/engine package version.

Compare supported stable SemVer versions numerically, not as strings. Apply this
policy before engine acquisition or repository writes:

| Repository pin compared with invoking tool | `init` | `init --force` |
| --- | --- | --- |
| Absent | Initialize | Initialize |
| Older version | Upgrade | Upgrade |
| Same version and revision | Revalidate; restore a missing verified cache if needed | Same |
| Same version, different revision | Error | Replace with the invoking tool's release |
| Newer version | Error | Downgrade to the invoking tool's release |

Revision ordering is not inferred from Git ancestry. An exact-identity retry
reuses the recorded reference without a mutable release lookup and does not
rewrite identical managed files. Other inconsistent identity or ownership data
is an error, not an exact-identity retry.

`--force` authorizes only the two otherwise-rejected version/revision transitions.
It does not disable SHA-256, embedded/sidecar SBOM, source identity, host,
compatibility, path safety, ownership or concurrent-change checks. It cannot
invent a missing release, rewrite public tags/assets, repair an untrusted pin by
guessing its identity, or overwrite consumer edits. Historical tools do not gain
this interface retroactively; unsupported reference/protocol migrations remain
errors, including with `--force`. Legacy bundles remain retired.

Errors show the installed and pinned version/revision and the reason for refusal.
Where the only conflict is an allowed override, explain the `init --force` option.
A successful update reports the previous and selected identities and whether an
upgrade, downgrade or same-version replacement occurred.

### Preserve product ownership and the previous working engine

Update the engine, its fixed reference and the standards-managed entry, local
Skill and AGENTS common block together. Prove that generated content still matches
its previous managed baseline before replacing it. If that ownership cannot be
established, or the consumer edited it, report a conflict requiring adaptation;
`--force` does not grant ownership of those edits.

Preserve existing product configuration, Make recipes, AGENTS repository
contracts and unrelated files. Do not edit Cargo/npm dependency declarations or
lockfiles. Report that common-crate Git revisions must follow the selected full
revision, and that configuration/API adaptation and product checks remain adoption
work. Initialization or an engine update does not certify product conformance.

Retain the exclusive init lock, owned staging, path checks and optimistic input
rechecks. Acquire and verify the selected engine before publishing managed
replacements, publish the pin last, and restore still-owned prior bytes on failure
without overwriting concurrent changes. A failed update preserves the previous
pin and valid engine cache; verification failure must not activate a replacement.
Keep cache entries distinct by fixed identity/reference and host so that the old
valid engine is not overwritten during acquisition.

This is a technical file operation, not a durable business lifecycle. The fixed
reference and version-controlled managed changes retain provenance; staging and
cache identities define the update boundary. Rejected and identical attempts are
not new state transitions. No business transition-history table or Header-Detail
aggregate is introduced (design-baselines).

### Keep setup and ordinary operations pinned

`make setup` acquires only the already-fixed engine; it never upgrades or selects
Latest. A person cloning an initialized repository needs the committed entry and
explicit setup, not a global initializer installation.

Build, check, release, install, package and prime continue to use the verified
repository pin, with no automatic download or update. Prime stays offline and
read-only and renders the fixed engine's embedded rules. Update the local Skill
and guidance to explain this distinction and the manual dependency follow-up.

## Consequences

The entry point for adopting or updating standards is one explicit `init` command.
Ordinary operations remain reproducible at the repository pin, regardless of the
globally installed initializer version. A forced downgrade is possible when the
target release and reference are supported, but is never an implicit side effect.

The CLI split changes distribution and direct global command usage. Implementation
must align installer generation, build/package receipts, release metadata, shared
entry dispatch, documentation and CI. Preserve artifact-specific consumer SBOMs,
local-only source integrity ledgers, standard Rust output paths and Rust 1.85
compatibility for public consumer libraries. ADR 0005's release policy is unchanged;
this breaking 0.x change does not itself authorize a 1.0.0 release.

## Acceptance and rollout

- Cover fresh init and both force modes across older, equal and newer versions,
  including equal-version revision conflicts and numeric ordering such as 0.9.0
  versus 0.10.0. Exact retries reuse the pin and avoid managed-file rewrites.
- Prove rejected transitions fail before transport or writes, and direct engine
  initialization applies the same guards as the public initializer.
- Verify both binary identities, exact embedded/sidecar SBOMs and host delivery
  metadata. Reject role substitution, missing assets and hash/revision/protocol
  mismatches even when `--force` is present.
- Exercise managed-entry updates, local-edit conflicts, concurrent changes and
  interrupted/failed acquisition or publication. Preserve the old working pin and
  cache on failed updates. Do not modify Cargo/npm inputs or product-owned files.
- Demonstrate init, explicit setup after a clean clone, offline prime and ordinary
  fixed-engine execution without a global full engine, legacy bundle or copied
  rules tree. Keep the common failure matrix in standards, not each consumer.
- Use isolated roots, temporary destinations and transport stubs; never live
  services or user state in tests. Run required checks, Rust 1.85 consumer checks,
  available-host acceptance and implementation-only code-size assessment when
  implementing. Record unexecuted hosts separately.

Implementation is tracked in bd as `standards-2bi`. Source qualification and
unexecuted hosts are recorded in the [verification guide](../guides/verification.md#adr-0006-source-qualification-2026-11-07).
This does not qualify a published release. Product adoption, real-user installation,
stage/commit, release preparation, tag, push, remote CI execution and publication
remain outside this implementation request.
