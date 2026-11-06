# ADR 0005: Explicit major release control

Status: Accepted; implemented in source

This replaces the automatic 0.x MAJOR increment rule in the current
[release contract](../../spec/releases.md), [release design](../design/releases.md)
and [release guide](../guides/releases.md). Those documents and the engine are aligned in source; see the
[qualification record](../guides/verification.md#adr-0005-source-qualification-2026-11-06). The published v0.3.0 engine still applies its old
rule; this decision does not change that binary. Implement this before the breaking
bundle replacement in [ADR 0004](0004-repository-initialization-and-prime.md).

## Context

The current engine turns a breaking Conventional Commit into a MAJOR increment
even when the current major version is zero. A breaking change to standards 0.3.0
would therefore select 1.0.0 without an explicit decision that the project is ready
for a stable API. Removing bundle distribution exposes that problem.

CraftGuild treats 0.x as development before an explicitly chosen stable release.
Breaking changes may ship there, but must not implicitly end that development
period. After 1.0.0, ordinary SemVer major increments apply.

## Decision

### Apply one common policy to every release group

Use the current version of the selected release group, not the standards engine's
version, to determine the increment. No per-project opt-in flag is needed.

| Current version | Scoped changes | Normal preparation |
| --- | --- | --- |
| 0.x | fix | PATCH, for example 0.3.0 to 0.3.1 |
| 0.x | feat | MINOR, for example 0.3.2 to 0.4.0 |
| 0.x | breaking marker or footer | MINOR, for example 0.3.2 to 0.4.0 |
| 1.0.0 or later | fix | PATCH |
| 1.0.0 or later | feat | MINOR |
| 1.0.0 or later | breaking marker or footer | MAJOR, for example 1.2.3 to 2.0.0 |
| Any version | only non-bumping changes | No normal post-tag release edits |

Recognize the existing `!`, `BREAKING CHANGE` and `BREAKING-CHANGE` forms. Retain
breaking descriptions and commit identities in the generated notes even when the
effective increment is MINOR. The rule changes version selection, not the meaning
or visibility of a breaking change.

Apply the greatest effective increment once across the selected range. A mix of
breaking, feature and fix commits does not cause several increments. Normal 0.x
preparation never crosses to major version one, including from 0.99.x. Invalid
messages, unsupported versions and arithmetic overflow remain errors rather than
an excuse to guess an increment or cross the stability boundary.

### Request a MAJOR increment explicitly

Add `--major` to the existing release interface:

    craftguild-standards release [--group ID] --major [--json]
    craftguild-standards release [--group ID] --major --apply

The corresponding Make forms are:

    make release RELEASE_ARGS="--major --json"
    make release RELEASE_ARGS="--major --apply"

The flag increments the selected group's current major version by one and resets
minor and patch to zero, regardless of the increment inferred from history. Thus
0.3.0 becomes 1.0.0, 1.2.3 becomes 2.0.0 and 2.5.1 becomes 3.0.0. This is a
general explicit MAJOR request, not a flag limited to stable promotion and not an
arbitrary-version override. Apply it once, even if the history also contains a
breaking change; do not increment again for that inferred change.

The explicit request can prepare a release with only fixes, non-bumping changes
or an empty post-tag range. It overrides increment selection, not history
validation: selected commits still need valid Conventional Commit messages and
OpenPGP signatures, and full history/tags and all other preparation guards remain
required. An explicit request never authorizes skipping those checks.

Without a previous tag, normal preparation retains initial_version and includes
history from the root as before. Explicit `--major` instead applies the same
increment to the configured initial version. A group deliberately configured to
start at initial_version = "1.0.0" already makes an explicit stable initial-version
choice; a normal first release there is not an automatic promotion from an existing
0.x release.

Require `--major` for both preview and apply, and on retries of that preparation.
The flag does not implicitly apply changes. Neither preparation form commits,
tags, pushes or publishes; the existing separate operational authorization still
applies. This does not add alpha/beta channels or SemVer prerelease/build metadata
support: 0.x here means ordinary versions with major zero.

### Preserve intent and replay safety

Record normal preparation versus explicit MAJOR in the immutable preparation
receipt and report the effective version increment in preview/JSON output. A
normal command must not apply or resume an explicit-MAJOR preparation; an explicit
MAJOR command must not reinterpret an existing normal preparation. Report a mode
conflict before writes and require finishing or explicitly reverting the existing
preparation through its own mode.

Rerunning the matching mode verifies exact prepared hashes and returns the same
target version, including after committing only the prepared edits. It must not
increment twice or regenerate duplicate notes. Historical receipts without an
explicit MAJOR intent cannot authorize an implicit transition to 1.0.0.

Retain all existing guards: explicit group ownership, synchronized Cargo/npm
versions and lock roots, full history/tags, complete revision endpoints, valid
OpenPGP signatures, clean/unrelated-dirty protection, changed-input checks,
exclusive preparation, rollback and existing-tag refusal. No dependency upgrades
or unrelated source edits belong to a version-only preparation. Signing or
verification failure retains the existing stop contract.

This remains the technical lifecycle described by the release design. Immutable
plans and receipts retain intent and preparation provenance; signed Git history
records actual publication. No business state/history tables or Header-Detail
records are introduced.

## Alternatives and consequences

Leaving automatic MAJOR increments in place would make a commit message decide
readiness for stable API commitments. Treating breaking changes as PATCH would
hide their intended release boundary. MINOR increments during 0.x preserve a
visible boundary while reserving 1.0.0 for an explicit decision. A general
`--major` flag also supports deliberate major boundaries after 1.0.0, even when
the selected history alone would not request one.

A configurable policy for each repository would allow identical 0.x histories
to imply different stability transitions. The owner chooses one shared policy
for all groups. Independent groups still use their own versions and release
scopes; requesting MAJOR for one group does not affect another.

The classifier still detects breaking changes. Consumers of preview/receipt data
must distinguish that fact from the effective increment and explicit MAJOR intent.
Current specifications stating that 0.x follows the same MAJOR rule must be
updated in the implementation change, not silently left as competing authorities.

## Acceptance and rollout

- Cover 0.x fix, feat, every breaking syntax, mixed ranges, non-bumping changes,
  0.0.x and high minor versions. No normal 0.x path selects 1.0.0.
- Cover explicit MAJOR preview/apply from both first-release and tagged states,
  before and after 1.0.0, including fix-only, non-bumping and empty post-tag ranges.
  Preserve explicitly configured initial versions in normal mode and prove that
  explicit MAJOR is applied once even when history already requests MAJOR.
- Cover independent release groups with different current major versions.
  Preserve normal PATCH/MINOR/MAJOR behavior for every group already at 1.0.0
  or later, including 2.x and subsequent major versions.
- Prove matching-mode replay before and after a preparation commit; reject both
  directions of mode mixing without changing files. Preserve tag collision,
  dirty-input, stale-hash, ownership and signature-failure coverage.
- Verify synchronized Cargo/npm edits, accurate preview/JSON intent and effective
  increment, and retained breaking descriptions/full commit identities.
- Use isolated signed Git fixtures and test-only keys. Run focused tests, required
  make check and code-size assessment for implementation changes. Do not invoke
  release preparation, signing or publication against this real repository to
  test the new rule.

The implementation prompt is registered in bd as `standards-a0c`; the ADR 0004
goal `standards-g6b` depends on it. Align the engine, canonical specs, guides,
source skill and tests together. Registration does not start implementation. Historical
versions, tags, notes, assets and receipts remain unchanged. Actual release
preparation, commits, tags, pushes and publication require a separate request.
