# Architecture decisions

- [0001: Artifact identity and offline presentation](0001-artifact-identity-and-offline-presentation.md) — Accepted.
- [0002: Build orchestration in standards](0002-standards-owned-build-orchestration.md) — Accepted.
- [0003: Standard Rust outputs and release installers](0003-standard-rust-outputs-and-release-installers.md) — Accepted; implemented in source; bundle additions superseded by ADR 0004.
- [0004: Repository initialization and prime](0004-repository-initialization-and-prime.md) — Accepted; implemented in source; replaces bundle distribution in ADR 0002/0003.
- [0005: Explicit major release control](0005-explicit-major-release-control.md) — Accepted; implemented in source; replaces automatic 0.x MAJOR increments and adds explicit MAJOR requests.
- [0006: Initializer CLI and controlled engine upgrades](0006-initializer-cli-and-controlled-engine-upgrades.md) — Accepted; implemented in source; supersedes ADR 0004's single CLI and existing-pin retention policy, with explicit force overrides.
