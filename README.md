# Craftguild standards

Shared CLI presentation, artifact SBOM preparation, quality checks and development
contracts for Craftguild products. Language-neutral rules are backed by Rust and
npm adapters and the `craftguild-standardize` agent skill.

This source branch also implements the unreleased build execution protocol for
ADR 0002. Its schema 2 configuration and entry bundle are separate from published
v0.1.0; package versions have not changed.

## Quick start

With the [build tools](#build-requirements) available, build from source:

```sh
git clone https://github.com/craftguild/standards.git
cd standards
make
```

The build prints the target triple and prepares a release executable under
`target/<target>/release/craftguild-standards`. Run that executable with `--help`,
`--version`, `banner product`, or `sbom`. The SBOM is embedded and prints as
CycloneDX 1.5 JSON without a banner.

## System requirements

The built CLI's help, version, banner and SBOM operations require no language
runtime, Git, network, configuration or daemon. Build and quality operations need
the appropriate build tools and a repository checkout.

The consumer libraries `craftguild-cli`, `craftguild-cli-build` and
`craftguild-core` declare Rust 1.85 compatibility. The standards executable uses
a separate toolchain requirement. Platform gates and evidence requirements are in the
[verification guide](docs/guides/verification.md).

## Build requirements

- Rust/Cargo 1.95 for the standards tool; Git; Make; Python 3 for repository scripts.
- tokei **15.0.0** with JSON serialization for quality checks.
- npm and a project's configured bundler/type/lint/test tools for npm distributions.
- GnuPG 2 with Ed25519 support for signed Git conformance fixtures.

Run `npm ci` in this repository before `make test` or `make check`; the npm adapter
uses the pinned TypeScript parser. The Web examples have their own lockfiles and
need `npm ci` in their frontend directories before their checks/builds.

Small Slant font data and CycloneDX schemas are pinned and included. No Node.js
process or font download is needed to generate a banner. Dependencies are locked
in Cargo.lock. `make install` only copies a built binary and supports `PREFIX`,
`BINDIR` and `DESTDIR`; its default destination is `/usr/local/bin`.

## Usage

```sh
craftguild-standards banner standards --output assets/banner.txt
craftguild-standards banner standards --check assets/banner.txt
craftguild-standards build --artifact standards
craftguild-standards build --artifact standards --profile debug
craftguild-standards check --json
craftguild-standards release --json
```

Configure artifacts, source roles, language adapters and repository checks in
[`craftguild.toml`](craftguild.toml). Read the [contracts](spec/README.md),
[architecture](docs/design/architecture.md), [configuration](docs/design/configuration.md)
and [skill](skills/craftguild-standardize/SKILL.md).
The source-development [execution guide](docs/guides/orchestration.md) explains
the pinned Make/launcher bundle, hooks, receipts and copy-only installation.
Use `make release RELEASE_ARGS=--json` to preview version updates and generated
notes, then `make release` to prepare them locally. See the
[release guide](docs/guides/releases.md) for groups, signed history and publication.
Common crates use fixed full Git revisions from this repository and are not
published to crates.io. A product retains its own argument grammar and passes
`Brand`, `BuildInfo` and embedded SBOM bytes to the output functions.
See the [standalone Rust](examples/rust-only/README.md),
[embedded Web](examples/embedded-web/README.md) and
[independent frontend/backend](examples/independent/README.md) examples.
Use `check --changed --json` after implementation edits to narrow source measurement.

## Known limitations

- The npm adapter consumes esbuild metafiles and package-lock v3. Other bundlers
  need an explicit adapter/evidence conversion; package discovery alone cannot
  establish which code shipped.
- Rust dependency reporting is package-granular. It follows the selected Cargo
  runtime graph, excluding development/build/procedural-macro edges; it does not
  claim symbol-level linker reachability.
- SeaORM inspection parses syntax and macro token identifiers. It does not perform
  compiler name/type resolution or expand arbitrary external macros.
- Unsupported conditional test separation is reported as unmeasured.
- Entry bundles contain a host-specific engine. Other hosts require a matching
  verified bundle; ordinary builds do not install toolchains or SDKs.
- Release acceptance and publication evidence remain requirements until recorded
  against the final source; a local focused test is not a platform certification.

## Community

Use repository issues for ordinary reproducible bugs. Start with the
[documentation](docs/README.md) and include the standards version, command and
relevant rule ID. Keep credentials and private product data out of reports.

## Security

No private vulnerability-reporting channel has been established for this project.
Do not publish sensitive vulnerability details in public issues.

## License

First-party code is licensed under [MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE). See the [font notice](assets/fonts/NOTICE.md) and
[schema notice](schemas/NOTICE.md) for third-party material.
