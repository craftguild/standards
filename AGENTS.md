<!-- craftguild:common:start -->
Use Craftguild standards 0.1.0. Read [the standards](spec/README.md),
[the configuration](docs/design/configuration.md), and the packaged
[craftguild-standardize skill](skills/craftguild-standardize/SKILL.md).
Run `make check`; use focused tests during development.
<!-- craftguild:common:end -->

## Repository contracts

The [architecture](docs/design/architecture.md) defines dependency directions.
Keep public consumer libraries compatible with Rust 1.85. Toolchain requirements
are separate. Do not publish crates to a registry.

Work only in this repository. Existing products are read-only references during
stage one. Stop after standards publication; product adoption needs a new request.
Preserve unrelated changes and never use live services or user state in tests.
