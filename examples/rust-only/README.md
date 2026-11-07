# sample

A CraftGuild standards conformance sample for a standalone Rust executable.

## Quick start

From the standards checkout root, with the default native output directory:

```sh
cargo fetch --locked
make build
standards_engine="$(pwd)/target/$(rustc -vV | sed -n 's/^host: //p')/release/craftguild-standards-engine"
cd examples/rust-only
cargo fetch --locked
make STANDARDS="$standards_engine"
make acceptance
```

Pass the same `STANDARDS` value to later Make commands that invoke the engine.
These fixtures use it to select the full engine directly. The installed
init-only CLI cannot run their build/check commands. Normal product adoption uses
the [generated pinned entry](../../docs/guides/orchestration.md).

## System requirements

The resulting Rust executable's help, version and SBOM need only the binary.

## Build requirements

Rust 1.85-compatible consumer libraries, a Rust toolchain for the selected target,
Make and Python 3. Building the standards tools requires Rust 1.95; the normative
baseline remains 0.1.0.
The fixture Cargo manifests pin `266b14893251e5fc0ec5879f345ec64cfc0cee70` over HTTPS.
For product adoption, use the full revision reported by init and update dependency
declarations manually; the fixture pin is not an engine upgrade policy.

## Usage

`sample --version` prints build facts; `sample sbom` prints exact embedded
CycloneDX JSON; `--machine-version` preserves a separate JSON path. `make check`
executes the configured shared and language checks. `make install` only copies an
existing release binary and supports PREFIX, BINDIR and DESTDIR.

## Known limitations

This sample demonstrates artifact composition and offline presentation. It has no
product service or persistent state.

## License

MIT OR Apache-2.0, as declared in the Cargo manifest. Third-party npm components
retain their package licenses. The font attribution is in the standards repository.

In a standalone Git checkout of this sample, use `make release RELEASE_ARGS=--json`
to preview a release and `make release` to prepare local version/notes changes.
Commit history must use Conventional Commits with verified OpenPGP signatures.
