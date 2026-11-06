# Build execution

The source engine implements ADR 0004 fixed-reference initialization and prime.
Published v0.3.0 and earlier releases retain their old delivery contracts; they
cannot initialize this workflow. A release implementing this source is required
for published adoption. Do not pin an older asset to unpublished source.

In the selected repository's root, use a clean published implementing tool:
Python 3 and curl must be available for initialization/setup. After setup, prime
needs Python 3 and the verified cache, with no network or compiler.

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
Explicit `make setup` restores a missing cache. Build, prepare, test/check,
release, install, package and prime never download, compile a replacement engine,
resolve Latest or consult sibling checkouts. Invalid caches fail closed. Hash,
revision, host and compatibility validation precede operational execution.

Prime is read-only offline Markdown from the pinned engine's embedded documents,
even when the invoking installed tool differs. Missing, incomplete or malformed
product config is reported as preparation status, not successful conformance.
Topics and full-revision source links need no copied rules tree.

After configuration, use the common operations:

```sh
make build PROFILE=debug
make build TARGET=x86_64-unknown-linux-gnu CARGO_TARGET_DIR=products
python3 standards/launcher.py build --artifact example-cli --profile release
make check
make acceptance
make release RELEASE_ARGS="--major --json"
```

The artifact name and target here are examples requiring real product declarations
and installed toolchains. Selectors, bounded argv hooks, operation preparation,
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
