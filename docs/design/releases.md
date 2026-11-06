# Release preparation

`release` is a local technical preparation operation. It never commits, creates
tags, pushes, or invokes GitHub. Preview is the default; `--apply` performs the
reviewed version and notes edits. Git commits use Conventional Commits 1.0.0;
normal release versions follow SemVer 2.0.0. Prerelease channels are deliberately
unsupported and produce an error, rather than guessing a stable increment.

Each configured release group owns explicit Cargo/npm version manifests,
corresponding lockfiles, path scope, initial version, tag prefix and notes directory.
The first Cargo/npm manifest is the version authority. All other configured
versions must agree. A workspace declaration owns only the explicitly listed
local packages; dependency versions and independent frontend groups are separate.
Workspace members must inherit their workspace version. Registry/Git dependencies
are never versioned as local packages. npm updates only the package and lock root,
not dependency versions. Unknown or inconsistent ownership fails before writes.

The baseline is the greatest reachable stable SemVer tag with the group's prefix.
All tags must be available (use a full clone/fetch). The range ends at the full
HEAD observed when planning. Literal repository paths select group commits;
merges with nonconventional messages are errors. Every selected commit must have
a valid OpenPGP signature. A breaking `!` or uppercase breaking footer wins over
`feat`, which wins over `fix`; other types do not increment. For a selected group at 0.x, breaking changes select an effective MINOR
increment; from 1.0.0 onward they select MAJOR. Normal 0.x releases never cross
to 1.x and arithmetic overflow fails. With no previous tag, normal preparation
uses the explicit initial version unchanged and all selected history is included. An empty post-tag range or only
non-bumping changes produces no edits.

Preparation requires a clean checkout, then fixes the comparison revisions,
versions, ordered notes, exact file edits and their before/after SHA-256 hashes.
The preview includes the notes and affected paths. Apply rechecks HEAD and input
hashes, holds an exclusive repository-local preparation lock, stages replacements,
and replaces files with rollback on I/O failure. The receipt is written last.
Unrelated dirty files are rejected. Notes and the receipt are versioned alongside
the manifests. The receipt records technical provenance, not secrets or timestamps.

A receipt identifies an untagged prepared release. Rerunning verifies the exact
prepared hashes and allows only its planned edits since its original HEAD, then
reports the existing preparation without incrementing again. New source changes
while preparation is pending fail; finish or explicitly revert the pending edits
before planning again. Tags are never overwritten. After the release is signed,
verified and tagged, that tag becomes the next baseline. Build the final SBOM and
binary from the signed clean release commit.

This applies the design-baselines skill to a technical lifecycle: immutable plans
and receipts preserve preparation provenance; signed Git commits/tags preserve
publication history. There is no durable business entity/current-state table to
update and no business line-item aggregate. File-edit arrays are technical
inventories, so no artificial Header-Detail database model is introduced. The
write boundary is a guarded, recoverable local file operation, not publication.


Explicit `release --major` selects current major + 1 with minor and patch zero,
including first releases (using `initial_version`), empty post-tag ranges and
non-bumping history. It overrides increment selection only: message/signature,
full-history, ownership, dirty-input and tag protections still apply. Breaking
explanations and full commit identities remain in notes. Use `--major` for preview,
apply and every retry, for example `make release RELEASE_ARGS="--major --json"`
then `make release RELEASE_ARGS="--major --apply"`. Receipt schema 2 records
`mode` (`normal` or `explicit-major`) and the effective `bump`; initial normal
preparation has no increment. Conflicting pending modes fail before writes.
Historical receipts do not authorize an implicit 0.x to 1.x transition.
