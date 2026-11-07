# Build execution

The installed `craftguild-standards` initializes and updates repositories.
The fixed `craftguild-standards-engine` handles their operations. This workflow
uses the split CLI introduced in v0.5.0. Historical v0.4.0 retains its single CLI;
earlier bundle-based references require explicit migration.

## Initialize a repository

First [install the initializer](installers.md#download-installation). In the
selected repository's root, Python 3 and curl must be available. Put the installed
CLI on PATH or call it by its installed path. Prime then needs Python 3 and the
verified cache, with no network or compiler.

```sh
craftguild-standards init
make prime
make prime PRIME_ARGS="--topic configuration"
```

Init preserves existing config, product contracts and Make recipes. When a
Makefile already exists, wire `include standards/make/common.mk` and its targets
explicitly without replacing product recipes. The generated starter is valid
TOML with no artifacts; configure the product before build/check. See the
[initialization design](../design/initialization.md) for ownership and rollback.
No Git initialization, dependency migration or global skill installation occurs.

Commit the fixed reference, thin entry, local skill and product configuration.
The entry selects only exact pinned host assets in `.craftguild/engine/`.
Explicit `make setup` acquires a missing cache. Build, prepare, test/check,
release, install, package and prime never download, compile a replacement engine,
resolve Latest or consult sibling checkouts. Invalid caches fail closed. Hash,
revision, host and compatibility validation precede operational execution.

Prime is read-only offline Markdown from the pinned engine's embedded documents,
independently of the installed initializer version. Missing, incomplete or malformed
product config is reported as preparation status, not successful conformance.
Topics and full-revision source links need no copied rules tree.

## Start after cloning

A clone includes the pin and common entry, but `.craftguild` is ignored. No global
initializer is needed:

```sh
make setup
make prime
```

Setup downloads only the version already fixed in `standards/engine.json`. It does
not update the pin or resolve Latest. Invalid existing cache data is an error;
do not bypass verification or substitute an installed engine.

## Update the fixed engine

Install the desired published initializer and run `craftguild-standards init` in
the repository root. It selects its own exact version and full revision.
Upgrades use numeric stable SemVer; `init --force` additionally permits downgrades
and same-version revision replacement. It never overrides consumer edits, invalid
hashes or unsupported protocols. Exact retries revalidate without mutable lookup
or rewriting managed files. Manually align common-crate Git revisions with the
reported revision, adapt config/APIs and run product checks. Setup never updates.
Missing or edited managed files require explicit adaptation. `--force` does not
overwrite those edits or accept an unsupported reference. See the complete
[update policy](../design/initialization.md#update-policy).

## Build and check

After configuration, use the common operations:

```sh
make build PROFILE=debug
make build TARGET=x86_64-unknown-linux-gnu CARGO_TARGET_DIR=products
python3 standards/launcher.py build --artifact example-cli --profile release
make check
make acceptance
make release RELEASE_ARGS=--json
```

The artifact name and target here are examples requiring real product declarations
and installed toolchains. Use `--major` only for an explicit major release request.
Selectors, bounded argv hooks, operation preparation,
receipts and dependency evidence follow the [execution contract](../design/orchestration.md).
New Rust adoption uses `<target-dir>/<target-triple>/<profile>/<file>` for native
and cross outputs. Products retain SDK, ABI, generator and domain gates; common
engine/font/measurement tests remain in standards. No font-renderer runtime or
development dependency is required merely to adopt the common entry.

Source-build `make install` copies only a verified output and honors PREFIX,
BINDIR and DESTDIR. Download installation has a separate
[installer contract](installers.md). Neither route is authorization to alter a
real user/system destination. Product adoption and actual publication remain
separate requests; report host qualification only for executions actually run.
