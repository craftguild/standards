# embedded-demo

A self-contained Craftguild standards conformance sample for Rust with embedded Web assets.

## Quick start

Build the standards tool first and put its release executable on PATH. From this
sample's root, install npm inputs with `npm --prefix web ci` before building.

```sh
make
make acceptance
```

## System requirements

The resulting Rust executable's help, version and SBOM need only the binary.

## Build requirements

Rust 1.85 libraries, a Rust toolchain for the selected target, Make, Python 3,
and standards 0.1.0. Node.js/npm are needed to build and check the frontend.
The consumer dependencies pin standards commit `76800657797708eaba16fc631a6d83d231557292` over HTTPS.

## Usage

`embedded-demo --version` prints build facts; `embedded-demo sbom` prints exact embedded
CycloneDX JSON; `--machine-version` preserves a separate JSON path. `make check`
executes the configured shared and language checks. `make install` only copies an
existing release binary and supports PREFIX, BINDIR and DESTDIR.

## Known limitations

This sample demonstrates artifact composition and offline presentation. It has no
product service or persistent state.

## License

MIT OR Apache-2.0, as declared in the Cargo manifest. Third-party npm components
retain their package licenses. The font attribution is in the standards repository.
