# Application workflow

In an initialized consumer read matching contracts through `make prime` and
`make prime PRIME_ARGS="--topic NAME"`. The fixed reference selects the engine's
embedded documents; source links include its full revision. In the standards
checkout the canonical authority is [`spec/`](../../../spec/README.md).

## Investigate and map differences

Record the requested scope and current design authorities. Inspect public command
names, early help/version paths, existing machine formats, package manager inputs,
embedded assets, generators, source roles and all required checks. Map each current
gate to an adapter, common policy or retained repository command before removal.
Keep immutable migrations, generated ABI/ownership checks and operational contracts.

Preserve the product guarantees each check protects, not obsolete orchestration
or internal paths merely because existing tests assert them. When a local adapter
is removed, update or remove tests of that adapter together and identify where its
required coverage now lives. Do not keep the adapter just to keep its tests.

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

## Build execution

For authorized adoption, use `craftguild-standards init` from the explicit
product root, then prime and finish real configuration. An existing Makefile
requires explicit shared-include wiring; preserve its recipes. Commit the small
reference/entry, local skill and config. Only init and explicit `make setup`
acquire verified pinned host assets. Ordinary operations fail with setup guidance
when cache is missing; no archive loader, sibling fallback or implicit bootstrap
is provided. New pins require a clean published implementing release. Published
v0.3.0 does not provide this workflow; never invent an unpublished consumer pin.

### Adopt the shared output layout

For Rust executables and libraries, use
`<target-dir>/<target-triple>/<profile>/<file>`, including the target triple for
native builds. Keep supported target-root and profile selectors. Update the
product's Make aliases, scripts, install/package consumers, tests and documentation
to the shared output location or verified receipt. Do not introduce an output
template, `output_root`, native-only `{target_path}`, copy-back, symlink or fallback
search solely to reproduce the former layout, such as `target/release/<bin>`.

Enable `rust_output_layout = "standard"` when supported by the selected pinned
engine. Check that engine's capabilities before adding configuration; the default
shared layout also works without this newer enforcement setting. A setting missing
from an older engine is not a reason to restore the historical path.

Retain a layout exception only for an explicit user requirement or a concrete
external contract, such as an SDK or loader's fixed artifact slot. Identify that
contract and keep the exception bounded. Existing files, tests or habits alone
are not evidence of a compatibility requirement. Frontend bundle layout retains
its own contract.

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
and receipts, outside consumer SBOM metadata.

### Keep verification with its owner

Standards owns tests of common context/path resolution, process cancellation and
rollback, seals and inventories, receipts, generic install/package behavior and
release preparation. Use the pinned version's existing conformance coverage and
recorded evidence; do not copy its fixtures or failure matrix into each consumer,
or rerun the full standards suite as a new consumer gate. Report a common gap or
regression upstream and add its test there when authorized. If coverage cannot be
confirmed, report that limit instead of claiming it passed.

Products retain domain, SDK, ABI, generator ownership/reproduction, hook and
distribution-specific guarantees. For the changed wiring, reuse existing artifact
acceptance to build the actual selected artifact, locate its verified output and
check offline presentation and exact embedded SBOM where applicable. Extend those
checks only for a gap introduced or exposed by this migration. Exercise common
failure paths in a consumer only when a product hook, driver or integration adds
an uncovered risk.

Before adding a test, name the concrete risk and explain why existing common and
product checks do not cover it. Prefer extending an existing focused check. Do
not add a general build-quality harness, duplicate configuration/flag matrices,
or tests of Make recipe text, private engine steps or old path strings merely
because standards is being adopted. Preserve meaningful existing coverage without
automatically preserving every old test implementation. Adding new suites and
running existing required gates are separate decisions; adoption alone does not
expand gate scopes or platform matrices.

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
each product. At 0.x breaking changes select MINOR; from 1.0.0 they select MAJOR. Explicit
`--major` requests current major + 1, including first releases, and must be
repeated for preview/apply/retry. It does not bypass validation. The matching
development/publication contracts define increment,
initial-version and repeat behavior. Sign every actual commit with the approved
OpenPGP key and verify it. Stop on signing/verification failure without bypassing
signing or continuing tag/push operations. Test histories use isolated test keys.

Follow the publication contract's reviewed inventory and `.private/oss-archive/`
procedure. Preserve working history locally; export to a separate directory and
create public history there. Publish dependencies first, then update full fixed
HTTPS revisions/provenance and freeze clean consumer commits. Confirm anonymous
fetch/build and actual CI before tagging. Do not begin this workflow merely
because a standardization request mentions public readiness.


## Package and qualify standalone installers

For an engine supporting the ADR 0003 source extension (not published v0.2.0),
declare `[[installers]]` with an executable artifact, GitHub repository, owning
release group and safe script output. Run the common `installer` command and its
read-only `--check`; ordinary `check` also rejects drift. Generated scripts remain
implementation code without a size exception. Do not add a consumer generator.

Use `package [--group ID] --output DIR` (or `make package PACKAGE_ARGS="..."`) after
clean release builds. All installer artifacts in the group must share target and
version, match its version authority and current full clean Git HEAD, and pass
complete receipt/seal/inventory/SBOM/output verification. Fresh staged output
contains versioned binaries, consumer SBOMs and target checksums. Neither package
nor launcher package dispatch may acquire an engine or start a build. Consumer
SBOMs and release assets exclude local source ledgers. Keep checksum files out of asset
maps and checksum inputs.

Download installers accept stable unprefixed `--version`, absolute `--bindir`,
`--uninstall` and `--help`. Destination priority is flag, BINDIR, then
HOME/.local/bin; source-build install keeps its /usr/local/bin/PREFIX/DESTDIR
contract. Latest selection pins one validated group tag; exact SHA-256 verification
precedes destination operations. Elevate only necessary destination operations,
retain the old binary on failure and remove only a regular command on uninstall.
Qualify shared failure matrices in standards with real sh/hash and network/sudo
process stubs. Run real binary temporary install/uninstall and offline identity/
exact SBOM acceptance on available hosts, recording unexecuted hosts separately.
