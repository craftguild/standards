# Application workflow

Read the matching packaged `standards/` contracts for CLI, distribution, quality,
development and publication. In the source repository these are the canonical
[`spec/` documents](../../../spec/README.md); release packaging copies those exact
files into this reference directory. The package manifest identifies their hashes.

## Investigate and map differences

Record the requested scope and current design authorities. Inspect public command
names, early help/version paths, existing machine formats, package manager inputs,
embedded assets, generators, source roles and all required checks. Map each current
gate to an adapter, common policy or retained repository command before removal.
Keep immutable migrations, generated ABI/ownership checks and operational contracts.

The three distribution patterns are Rust-only, Rust with embedded Web assets, and
independent frontend/backend. The backend's artifact omits the separate frontend;
embedded Web inclusion follows bundle input evidence, not all development packages.
Use explicit supplemental records for fonts/native assets and explain evidence.

## Reusable policies and product checks

See [the policy configuration example](policies.toml) for shared dependency,
SeaORM and generated-ownership wiring. Select only the product's actual policies.

Use a dependency policy's `layers` as exhaustive allowed first-party edges, and
`restricted` to limit a library to named layers. Select SeaORM policy only where
the product contract requires it; its source scope includes relevant tests.
The Rust adapter parses real syntax, including macro token identifiers, rather
than matching comment/string text. This is not compiler name/type resolution or
proof about arbitrary external macro expansion; retain product checks for those
contracts when necessary.

Generated groups bind an owner, complete hash inventory and canonical inputs to
an isolated reproduction command. Use `{output}` as a separate argv argument;
the checker supplies an empty temporary output directory and compares every file.
`rust_size_exception` only grants a size exception after all verification passes.
Canonical generated ownership formats can be converted by a small repository
adapter, preserving the actual generator/schema/config/ABI/input contracts.

Connect product invariants with a small command, for example:

```toml
[[commands]]
id = "frozen-migrations"
purpose = "Verify released migration source digests against the product ledger"
argv = ["python3", "scripts/check-migrations.py"]
phase = "check"
```

This repository command owns the migration list and product ledger. It does not
reimplement common source measurement, dependency or ORM logic. For ordinary
language checks, configure a Rust/npm adapter with concrete test/lint/format/type
commands. Missing tools or an empty argv are failures.

## Source-development execution

For a request that includes build orchestration, use the packaged
`assets/execution` entry and schema 2 declarations described by the
[execution contract](orchestration.md). Copy the complete verified entry, including
its descriptor and vendored bundle, into the selected repository. Use a real fixed
revision for released adoption or explicitly identify a source export for local
development. Keep toolchain requirements separate from consumer-library MSRV.

Map product operations to prerequisite/prepare/evidence/verify argv hooks with
declared inputs and fresh outputs. Let the engine own context resolution, assets,
dependency inclusion, seals, compilation, verification and receipts. Do not retain
a product Rust adapter or hide that common sequence in a hook. Preserve platform,
ABI, ownership/reproduction, language and public CLI gates. Configure ordinary
test/check preparation separately from explicit acceptance; cached ownership
conversion belongs under `.craftguild`, not in tracked source.

Use the shared resolver for build/install and retain public aliases/selectors.
Install copies only a matching verified output and must fail without acquiring
an engine or invoking a compiler. Keep source inventories/hashes in local seals
and receipts, outside consumer SBOM metadata. Verify failure/cancellation rollback,
offline help/version/machine JSON/exact SBOM, and release reapplication after a
build refresh before claiming completion.

## Apply and verify

Add a versioned `craftguild.toml`, short common AGENTS.md reference block and
repository-specific contracts. Other agent files reference AGENTS.md. Preserve
the accepted README outline and docs entry/layout. Keep actual ADR numbers,
statuses and supersession when moving files; update links and validators.

Verify focused changes first, then `make check` and applicable isolated acceptance.
Record measured scope, pinned counter/version, maximum Code value, unmeasured
files, exceptions, executed gates and unavailable platforms. Root help/version/sbom
must work from only the binary without tools, valid configuration or state writes.

## Publication preparation when requested

Configure explicit release groups, synchronized Cargo/npm version owners and
separate independent artifacts. Use the common `release --json` preview followed
by `release --apply` through `make release`; do not copy version/history logic into
each product. The packaged development/publication contracts define increment,
initial-version and repeat behavior. Sign every actual commit with the approved
OpenPGP key and verify it. Stop on signing/verification failure without bypassing
signing or continuing tag/push operations. Test histories use isolated test keys.

Follow the publication contract's reviewed inventory and `.private/oss-archive/`
procedure. Preserve working history locally; export to a separate directory and
create public history there. Publish dependencies first, then update full fixed
HTTPS revisions/provenance and freeze clean consumer commits. Confirm anonymous
fetch/build and actual CI before tagging. Do not begin this workflow merely
because a standardization request mentions public readiness.
