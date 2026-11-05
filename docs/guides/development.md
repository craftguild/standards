# Development

`make` uses a non-distributed bootstrap executable to prepare the final tool's
own real SBOM, then builds the locked release binary. The bootstrap has no final
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
cargo +1.85.0 check --locked -p craftguild-cli -p craftguild-cli-build
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

Package a host-specific, self-contained skill bundle after building:

```sh
python3 scripts/package_skill.py --output target/craftguild-standardize.tar.gz
```

The bundle contains the skill, exact normative rules, schemas, licenses, a hash
manifest, compatible entry bundle and final host executable. Packaging uses the
verified build receipt, including a selected profile/output directory. Extract it into the chosen agent's skill
directory and add its `assets/bin` directory to the command search path when using
the skill. Creating the bundle never installs or alters global skills.

Before publishing, review `python3 scripts/public_files.py`. An explicit
`--export` writes that inventory to a new directory for fresh public history.
Ignored private archives and task/connection data never enter that inventory.

For host release assets, first commit and verify the source, then run `make build`.
The delivery packager requires a clean full-revision native release build:

```sh
python3 scripts/package_delivery.py --output target/delivery
```

It creates the executable, consumer SBOM, matching entry/skill archives, release
metadata and checksums. It invokes existing receipt validation and never builds
or publishes. CI retains these assets for the exact tested commit; tagging and
publication remain separate authorized operations.
