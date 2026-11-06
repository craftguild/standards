# Development

`make` uses a non-distributed bootstrap executable to prepare the final tool's
own real SBOMs, then builds both locked release binaries. The bootstrap has no final
version/SBOM presentation path. `make test` runs ordinary Rust, npm and Python tests. `make check`
uses the common engine, roles, counter and configured adapters. `make acceptance`
needs an already-built release executable and uses only temporary resources.

Git conformance tests require GnuPG 2 with Ed25519 support. They create disposable
keyrings and test keys, sign each fixture commit with OpenPGP, verify the resulting
commit, and remove the keyring. They never use personal signing keys or modify
global Git/GPG settings. A signing or verification error fails the check.

Sign actual work with your approved OpenPGP key (`git commit -S`), then verify the
full revision with `git verify-commit`. Stop on signing or verification failure;
never bypass signing to continue committing, tagging or publishing. Use
Conventional Commits messages for release history.

For focused changes:

```sh
cargo test --locked -p craftguild-cli
cargo test --locked -p craftguild-standards --no-default-features --features bootstrap --test quality
python3 scripts/check_docs.py
```

Consumer-library MSRV checks:

```sh
cargo +1.85.0 check --locked -p craftguild-cli -p craftguild-core -p craftguild-cli-build
```

The tool's Rust 1.95 requirement does not change the libraries' Rust 1.85 contract.
Do not install the bootstrap as a product. Direct final `cargo build` refuses
missing or stale preparation. Release preparation intentionally differs from a
debug build. Use `make build PROFILE=debug` for matching debug identity and seals;
the default remains release. Never rewrite release preparation to reuse it as debug.

Install copies an existing binary without invoking compilers or npm:

```sh
make install DESTDIR=/tmp/craftguild-stage PREFIX=/opt/craftguild
make build TARGET=x86_64-unknown-linux-gnu CARGO_TARGET_DIR=/tmp/craftguild-target
make install TARGET=x86_64-unknown-linux-gnu CARGO_TARGET_DIR=/tmp/craftguild-target DESTDIR=/tmp/craftguild-stage
```

Cross-target builds need the actual target toolchain/linker. These commands do not
claim cross-compilation is available on every host. The install entry reads the
verified receipt and the same output resolver as build. It fails if selectors,
inputs, SBOM or binary differ, and never probes Rust/npm or bootstraps an engine.
Build/setup must first run without privileges. See the
[execution guide](orchestration.md) for compatible consumer entries.

Rules and adoption guidance are embedded, sealed inputs of the engine. Consumer
`init` generates a short local skill and entry; no skill archive or global skill
installation is produced. See the [initialization design](../design/initialization.md)
and [execution guide](orchestration.md). Tests use isolated release fixtures and
transport stubs, never unpublished revisions presented as real consumer pins.

Before publishing, review `python3 scripts/public_files.py`. An explicit
`--export` writes that inventory to a new directory for fresh public history.
Ignored private archives and task/connection data never enter that inventory.

For host release assets, first commit and verify the source, then run `make build`.
The delivery packager requires a clean full-revision native release build and
a clean checkout at that same revision:

```sh
python3 scripts/package_delivery.py --output target/delivery
```

It creates both flat executables, their independent consumer SBOMs, schema 3 host release
metadata and target checksums. Combine independently qualified host deliveries
with `python3 scripts/release_metadata.py INPUT --output target/release-index`
to produce the versioned index used by init. Only assets actually present are
listed; all hosts must share version, full revision and compatibility. It invokes existing receipt validation and never builds
or publishes. CI retains these assets for the exact tested commit; tagging and
publication remain separate authorized operations.

The source engine also supports [download installers](installers.md) and common
release assets. Ordinary Rust build resolution/compilation uses locked offline
Cargo inputs. Obtain dependencies explicitly before building (for example,
`cargo fetch --locked`); builds do not install toolchains, SDKs or npm dependencies.

CI creates a fresh job-local `CARGO_HOME` and adds its `bin` directory to PATH.
Its explicit acquisition phase fetches the workspace and every standalone example
before invoking offline bootstrap/build operations:

```sh
cargo fetch --locked
cargo fetch --locked --manifest-path examples/rust-only/Cargo.toml
cargo fetch --locked --manifest-path examples/embedded-web/Cargo.toml
cargo fetch --locked --manifest-path examples/independent/backend/Cargo.toml
```

Use the same preparation when checking examples from a new local Cargo cache.
Example Git pins are separate inputs; fetching only the workspace does not prepare
them. Ordinary build, install and package keep their no-acquisition behavior.
